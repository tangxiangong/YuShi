use crate::{
    cli::{QueueArgs, QueueCommands},
    config::Config,
    ui::{ProgressManager, format_size, print_info, print_success},
};
use anyhow::{Result, anyhow};
use console::style;
use std::path::PathBuf;
use yushi_core::{ChecksumType, DownloadQueue, Priority, QueueEvent};

pub async fn execute(args: QueueArgs) -> Result<()> {
    match args.command {
        QueueCommands::Add {
            url,
            output,
            priority,
            md5,
            sha256,
        } => add_task(url, output, priority, md5, sha256).await,
        QueueCommands::List => list_tasks().await,
        QueueCommands::Start {
            max_tasks,
            connections,
        } => start_queue(max_tasks, connections).await,
        QueueCommands::Pause { task_id } => pause_task(task_id).await,
        QueueCommands::Resume { task_id } => resume_task(task_id).await,
        QueueCommands::Cancel { task_id } => cancel_task(task_id).await,
        QueueCommands::Remove { task_id } => remove_task(task_id).await,
        QueueCommands::Clear => clear_completed().await,
    }
}

async fn add_task(
    url: String,
    output: PathBuf,
    priority_str: String,
    md5: Option<String>,
    sha256: Option<String>,
) -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, _) = DownloadQueue::new(4, 1, queue_path);

    // 加载现有队列
    queue.load_from_state().await?;

    // 解析优先级
    let priority = match priority_str.to_lowercase().as_str() {
        "low" => Priority::Low,
        "normal" => Priority::Normal,
        "high" => Priority::High,
        _ => return Err(anyhow!("无效的优先级: {}", priority_str)),
    };

    // 解析校验和
    let checksum = if let Some(hash) = md5 {
        Some(ChecksumType::Md5(hash))
    } else {
        sha256.map(ChecksumType::Sha256)
    };

    // 添加任务
    let task_id = queue
        .add_task_with_options(url.clone(), output.clone(), priority, checksum, true)
        .await?;

    print_success("任务已添加到队列");
    println!("  任务 ID: {}", style(&task_id).cyan());
    println!("  URL: {}", url);
    println!("  输出: {}", output.display());
    println!("  优先级: {:?}", priority);

    Ok(())
}

async fn list_tasks() -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, _) = DownloadQueue::new(4, 1, queue_path);

    queue.load_from_state().await?;
    let tasks = queue.get_all_tasks().await;

    if tasks.is_empty() {
        print_info("队列为空");
        return Ok(());
    }

    println!("\n{}", style("下载队列").bold().underlined());
    println!();

    for task in tasks {
        let status_str = match task.status {
            yushi_core::TaskStatus::Pending => style("等待中").yellow(),
            yushi_core::TaskStatus::Downloading => style("下载中").green(),
            yushi_core::TaskStatus::Paused => style("已暂停").blue(),
            yushi_core::TaskStatus::Completed => style("已完成").green(),
            yushi_core::TaskStatus::Failed => style("失败").red(),
            yushi_core::TaskStatus::Cancelled => style("已取消").red(),
        };

        println!("{} {}", style("●").bold(), status_str);
        println!("  ID: {}", style(&task.id[..16]).cyan());
        println!("  URL: {}", task.url);
        println!("  输出: {}", task.dest.display());
        println!("  优先级: {:?}", task.priority);

        if task.total_size > 0 {
            let progress = (task.downloaded as f64 / task.total_size as f64) * 100.0;
            println!(
                "  进度: {:.1}% ({} / {})",
                progress,
                format_size(task.downloaded),
                format_size(task.total_size)
            );
        } else {
            // 流式下载，只显示已下载量
            println!("  进度: {} (流式下载)", format_size(task.downloaded));
        }

        if task.speed > 0 {
            println!("  速度: {}/s", format_size(task.speed));
        }

        if let Some(eta) = task.eta {
            println!("  剩余时间: {}s", eta);
        }

        if let Some(error) = &task.error {
            println!("  {}: {}", style("错误").red(), error);
        }

        println!();
    }

    Ok(())
}

async fn start_queue(max_tasks: usize, connections: usize) -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, mut event_rx) = DownloadQueue::new(connections, max_tasks, queue_path);

    queue.load_from_state().await?;

    let tasks = queue.get_all_tasks().await;
    let pending_count = tasks
        .iter()
        .filter(|t| t.status == yushi_core::TaskStatus::Pending)
        .count();

    if pending_count == 0 {
        print_info("没有待处理的任务");
        return Ok(());
    }

    print_info(&format!("启动队列处理 ({} 个待处理任务)", pending_count));
    print_info(&format!("最大并发任务: {}", max_tasks));
    print_info(&format!("每任务连接数: {}", connections));
    println!();

    let progress_mgr = ProgressManager::new();

    // 事件处理
    let event_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                QueueEvent::TaskStarted { task_id } => {
                    println!("🚀 开始: {}", &task_id[..8]);
                }
                QueueEvent::TaskProgress {
                    task_id,
                    downloaded,
                    total,
                    speed,
                    ..
                } => {
                    progress_mgr
                        .update_progress(&task_id, downloaded, speed)
                        .await;
                    if total > 0 && downloaded == 0 {
                        progress_mgr.add_task(task_id, Some(total)).await;
                    } else if total == 0 && downloaded == 0 {
                        // 流式下载
                        progress_mgr.add_task(task_id, None).await;
                    }
                }
                QueueEvent::TaskCompleted { task_id } => {
                    progress_mgr.finish_task(&task_id, true).await;
                }
                QueueEvent::TaskFailed { task_id, error } => {
                    progress_mgr.finish_task(&task_id, false).await;
                    eprintln!("❌ 失败 {}: {}", &task_id[..8], error);
                }
                QueueEvent::VerifyStarted { task_id } => {
                    println!("🔍 校验: {}", &task_id[..8]);
                }
                QueueEvent::VerifyCompleted { task_id, success } => {
                    if success {
                        println!("✅ 校验通过: {}", &task_id[..8]);
                    } else {
                        println!("❌ 校验失败: {}", &task_id[..8]);
                    }
                }
                _ => {}
            }
        }
    });

    // 等待中断信号
    tokio::signal::ctrl_c().await?;
    println!("\n\n收到中断信号，正在停止...");

    event_handle.abort();
    print_success("队列已停止");

    Ok(())
}

async fn pause_task(task_id: String) -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, _) = DownloadQueue::new(4, 1, queue_path);

    queue.load_from_state().await?;
    queue.pause_task(&task_id).await?;

    print_success(&format!("任务已暂停: {}", &task_id[..16]));
    Ok(())
}

async fn resume_task(task_id: String) -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, _) = DownloadQueue::new(4, 1, queue_path);

    queue.load_from_state().await?;
    queue.resume_task(&task_id).await?;

    print_success(&format!("任务已恢复: {}", &task_id[..16]));
    Ok(())
}

async fn cancel_task(task_id: String) -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, _) = DownloadQueue::new(4, 1, queue_path);

    queue.load_from_state().await?;
    queue.cancel_task(&task_id).await?;

    print_success(&format!("任务已取消: {}", &task_id[..16]));
    Ok(())
}

async fn remove_task(task_id: String) -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, _) = DownloadQueue::new(4, 1, queue_path);

    queue.load_from_state().await?;
    queue.remove_task(&task_id).await?;

    print_success(&format!("任务已移除: {}", &task_id[..16]));
    Ok(())
}

async fn clear_completed() -> Result<()> {
    let queue_path = Config::queue_state_path()?;
    let (queue, _) = DownloadQueue::new(4, 1, queue_path);

    queue.load_from_state().await?;
    queue.clear_completed().await?;

    print_success("已清空所有已完成任务");
    Ok(())
}
