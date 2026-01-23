use anyhow::Result;
use std::path::PathBuf;
use yushi_core::{DownloadQueue, QueueEvent};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建下载队列
    // 参数1: 每个任务的最大并发下载数（分块下载）
    // 参数2: 队列中同时运行的最大任务数
    // 参数3: 队列状态持久化文件路径
    let (queue, mut event_rx) = DownloadQueue::new(
        4, // 每个文件使用4个并发连接下载
        2, // 同时下载2个文件
        PathBuf::from("queue_state.json"),
    );

    // 从之前的状态恢复（如果存在）
    if let Err(e) = queue.load_from_state().await {
        eprintln!("Failed to load queue state: {}", e);
    }

    // 启动事件监听器
    let event_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                QueueEvent::TaskAdded { task_id } => {
                    println!("✅ 任务已添加: {}", task_id);
                }
                QueueEvent::TaskStarted { task_id } => {
                    println!("🚀 任务开始下载: {}", task_id);
                }
                QueueEvent::TaskProgress {
                    task_id,
                    downloaded,
                    total,
                    speed,
                    eta,
                } => {
                    let speed_mb = speed as f64 / 1024.0 / 1024.0;
                    if total > 0 {
                        let progress = (downloaded as f64 / total as f64) * 100.0;
                        print!(
                            "📊 任务 {} 进度: {:.2}% ({}/{}) @ {:.2} MB/s",
                            &task_id[..8],
                            progress,
                            downloaded,
                            total,
                            speed_mb
                        );
                    } else {
                        // 流式下载，无法显示百分比
                        print!(
                            "📊 任务 {} 进度: {} bytes @ {:.2} MB/s (流式下载)",
                            &task_id[..8],
                            downloaded,
                            speed_mb
                        );
                    }
                    if let Some(eta_secs) = eta {
                        println!(" (ETA: {}s)", eta_secs);
                    } else {
                        println!();
                    }
                }
                QueueEvent::TaskCompleted { task_id } => {
                    println!("✨ 任务完成: {}", task_id);
                }
                QueueEvent::TaskFailed { task_id, error } => {
                    eprintln!("❌ 任务失败: {} - {}", task_id, error);
                }
                QueueEvent::TaskPaused { task_id } => {
                    println!("⏸️  任务暂停: {}", task_id);
                }
                QueueEvent::TaskResumed { task_id } => {
                    println!("▶️  任务恢复: {}", task_id);
                }
                QueueEvent::TaskCancelled { task_id } => {
                    println!("🚫 任务取消: {}", task_id);
                }
                QueueEvent::VerifyStarted { task_id } => {
                    println!("🔍 开始校验: {}", task_id);
                }
                QueueEvent::VerifyCompleted { task_id, success } => {
                    if success {
                        println!("✅ 校验通过: {}", task_id);
                    } else {
                        println!("❌ 校验失败: {}", task_id);
                    }
                }
            }
        }
    });

    // 添加下载任务
    println!("\n=== 添加下载任务 ===");

    let task1 = queue
        .add_task(
            "https://speed.hetzner.de/100MB.bin".to_string(),
            PathBuf::from("downloads/file1.bin"),
        )
        .await?;
    println!("任务1 ID: {}", task1);

    let task2 = queue
        .add_task(
            "https://speed.hetzner.de/100MB.bin".to_string(),
            PathBuf::from("downloads/file2.bin"),
        )
        .await?;
    println!("任务2 ID: {}", task2);

    let task3 = queue
        .add_task(
            "https://speed.hetzner.de/100MB.bin".to_string(),
            PathBuf::from("downloads/file3.bin"),
        )
        .await?;
    println!("任务3 ID: {}", task3);

    // 等待一段时间
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // 暂停第一个任务
    println!("\n=== 暂停任务1 ===");
    queue.pause_task(&task1).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 恢复第一个任务
    println!("\n=== 恢复任务1 ===");
    queue.resume_task(&task1).await?;

    // 查看所有任务状态
    println!("\n=== 当前任务状态 ===");
    let all_tasks = queue.get_all_tasks().await;
    for task in &all_tasks {
        println!(
            "任务 {}: 状态={:?}, 进度={}/{} ({:.2}%)",
            &task.id[..8],
            task.status,
            task.downloaded,
            task.total_size,
            if task.total_size > 0 {
                (task.downloaded as f64 / task.total_size as f64) * 100.0
            } else {
                0.0
            }
        );
    }

    // 等待所有任务完成
    println!("\n=== 等待任务完成 ===");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    // 清理已完成的任务
    println!("\n=== 清理已完成任务 ===");
    queue.clear_completed().await?;

    event_handle.abort();

    println!("\n✅ 所有任务处理完成！");

    Ok(())
}
