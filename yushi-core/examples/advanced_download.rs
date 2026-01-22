use anyhow::Result;
use std::path::PathBuf;
use yushi_core::{ChecksumType, DownloadConfig, DownloadQueue, Priority, QueueEvent, YuShi};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 YuShi 高级下载示例\n");

    // === 示例 1: 使用自定义配置的单文件下载 ===
    println!("=== 示例 1: 自定义配置下载 ===");
    example_custom_config().await?;

    // === 示例 2: 带优先级和校验的队列下载 ===
    println!("\n=== 示例 2: 优先级和校验 ===");
    example_priority_and_checksum().await?;

    // === 示例 3: 使用回调处理完成事件 ===
    println!("\n=== 示例 3: 完成回调 ===");
    example_with_callback().await?;

    Ok(())
}

/// 示例 1: 使用自定义配置下载
async fn example_custom_config() -> Result<()> {
    use tokio::sync::mpsc;
    use yushi_core::ProgressEvent;

    // 创建自定义配置
    let mut config = DownloadConfig {
        max_concurrent: 8,                  // 8 个并发连接
        chunk_size: 5 * 1024 * 1024,        // 5MB 分块
        speed_limit: Some(2 * 1024 * 1024), // 限速 2 MB/s
        user_agent: Some("YuShi-Example/1.0".to_string()),
        ..Default::default()
    };

    // 添加自定义 HTTP 头
    config
        .headers
        .insert("Accept".to_string(), "*/*".to_string());

    // 如果需要代理，取消下面的注释
    // config.proxy = Some("http://proxy.example.com:8080".to_string());

    println!("配置:");
    println!("  - 并发连接: {}", config.max_concurrent);
    println!("  - 分块大小: {} MB", config.chunk_size / 1024 / 1024);
    println!(
        "  - 速度限制: {} MB/s",
        config.speed_limit.unwrap_or(0) / 1024 / 1024
    );

    let downloader = YuShi::with_config(config);
    let (tx, mut rx) = mpsc::channel(1024);

    // 进度监听
    let progress_handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                ProgressEvent::Initialized { total_size } => {
                    println!(
                        "开始下载，文件大小: {:.2} MB",
                        total_size as f64 / 1024.0 / 1024.0
                    );
                }
                ProgressEvent::ChunkUpdated { .. } => {
                    // 这里可以计算进度，但为了简化示例，我们跳过
                }
                ProgressEvent::Finished => {
                    println!("✅ 下载完成!");
                }
                ProgressEvent::Failed(e) => {
                    eprintln!("❌ 下载失败: {}", e);
                }
            }
        }
    });

    // 执行下载（使用小文件进行测试）
    match downloader
        .download(
            "https://speed.hetzner.de/10MB.bin",
            "downloads/custom_config.bin",
            tx,
        )
        .await
    {
        Ok(_) => println!("下载任务提交成功"),
        Err(e) => eprintln!("下载失败: {}", e),
    }

    progress_handle.await?;
    Ok(())
}

/// 示例 2: 优先级和文件校验
async fn example_priority_and_checksum() -> Result<()> {
    let (queue, mut event_rx) = DownloadQueue::new(
        4, // 每个任务 4 个并发连接
        3, // 同时运行 3 个任务
        PathBuf::from("advanced_queue.json"),
    );

    // 事件监听
    let event_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                QueueEvent::TaskAdded { task_id } => {
                    println!("➕ 添加任务: {}", &task_id[..8]);
                }
                QueueEvent::TaskStarted { task_id } => {
                    println!("🚀 开始: {}", &task_id[..8]);
                }
                QueueEvent::TaskProgress {
                    task_id,
                    downloaded,
                    total,
                    speed,
                    eta,
                } => {
                    let progress = (downloaded as f64 / total as f64) * 100.0;
                    let speed_mb = speed as f64 / 1024.0 / 1024.0;
                    print!(
                        "\r📊 {}: {:.1}% @ {:.2} MB/s",
                        &task_id[..8],
                        progress,
                        speed_mb
                    );
                    if let Some(eta_secs) = eta {
                        print!(" (ETA: {}s)  ", eta_secs);
                    }
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
                QueueEvent::TaskCompleted { task_id } => {
                    println!("\n✅ 完成: {}", &task_id[..8]);
                }
                QueueEvent::VerifyStarted { task_id } => {
                    println!("\n🔍 校验中: {}", &task_id[..8]);
                }
                QueueEvent::VerifyCompleted { task_id, success } => {
                    if success {
                        println!("✅ 校验通过: {}", &task_id[..8]);
                    } else {
                        println!("❌ 校验失败: {}", &task_id[..8]);
                    }
                }
                QueueEvent::TaskFailed { task_id, error } => {
                    println!("\n❌ 失败: {} - {}", &task_id[..8], error);
                }
                _ => {}
            }
        }
    });

    // 添加高优先级任务（带 MD5 校验）
    println!("添加高优先级任务（带校验）...");
    let _high_priority = queue
        .add_task_with_options(
            "https://speed.hetzner.de/10MB.bin".to_string(),
            PathBuf::from("downloads/high_priority.bin"),
            Priority::High,
            Some(ChecksumType::Md5(
                "f1c9645dbc14efddc7d8a322685f26eb".to_string(),
            )), // 10MB.bin 的实际 MD5
            false,
        )
        .await?;

    // 添加普通优先级任务
    println!("添加普通优先级任务...");
    let _normal = queue
        .add_task_with_options(
            "https://speed.hetzner.de/10MB.bin".to_string(),
            PathBuf::from("downloads/normal.bin"),
            Priority::Normal,
            None,
            false,
        )
        .await?;

    // 添加低优先级任务
    println!("添加低优先级任务...");
    let _low = queue
        .add_task_with_options(
            "https://speed.hetzner.de/10MB.bin".to_string(),
            PathBuf::from("downloads/low_priority.bin"),
            Priority::Low,
            None,
            true, // 自动重命名
        )
        .await?;

    println!("\n等待任务完成...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    event_handle.abort();
    Ok(())
}

/// 示例 3: 使用完成回调
async fn example_with_callback() -> Result<()> {
    let (mut queue, mut event_rx) = DownloadQueue::new(4, 2, PathBuf::from("callback_queue.json"));

    // 设置完成回调
    queue.set_on_complete(|task_id, result| async move {
        match result {
            Ok(_) => {
                println!("\n🎉 回调: 任务 {} 成功完成!", &task_id[..8]);
                // 这里可以执行后续操作：
                // - 发送通知
                // - 解压文件
                // - 移动文件到其他位置
                // - 更新数据库
                // - 触发其他任务
            }
            Err(error) => {
                eprintln!("\n⚠️  回调: 任务 {} 失败: {}", &task_id[..8], error);
                // 错误处理：
                // - 记录日志
                // - 发送警报
                // - 重试逻辑
            }
        }
    });

    // 简单的事件监听
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                QueueEvent::TaskAdded { task_id } => {
                    println!("添加: {}", &task_id[..8]);
                }
                QueueEvent::TaskStarted { task_id } => {
                    println!("开始: {}", &task_id[..8]);
                }
                _ => {}
            }
        }
    });

    // 添加任务
    println!("添加测试任务...");
    let _task = queue
        .add_task(
            "https://speed.hetzner.de/10MB.bin".to_string(),
            PathBuf::from("downloads/callback_test.bin"),
        )
        .await?;

    println!("等待任务完成（回调将被触发）...");
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    Ok(())
}
