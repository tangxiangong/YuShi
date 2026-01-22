//! YuShi - 高性能异步下载库
//!
//! 提供单文件下载和队列管理功能，支持断点续传、并发下载等特性。

pub mod nbyte;

mod downloader;
mod queue;
mod state;
mod types;

// 重新导出公共 API
pub use downloader::YuShi;
pub use queue::DownloadQueue;
pub use types::{DownloadTask, ProgressEvent, QueueEvent, TaskStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_download() -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1024);
        let downloader = YuShi::new(4);

        // 观察者线程
        tokio::spawn(async move {
            let mut total = 0;
            let mut current = 0;
            while let Some(event) = rx.recv().await {
                match event {
                    ProgressEvent::Initialized { total_size } => total = total_size,
                    ProgressEvent::ChunkUpdated { delta, .. } => {
                        current += delta;
                        println!("Progress: {:.2}%", (current as f64 / total as f64) * 100.0);
                    }
                    ProgressEvent::Finished => println!("Done!"),
                    ProgressEvent::Failed(e) => eprintln!("Error: {}", e),
                }
            }
        });

        downloader
            .download("https://speed.hetzner.de/100MB.bin", "video.mp4", tx)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_download_queue() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let queue_state_path = temp_dir.join("test_queue_state.json");

        // 清理之前的测试文件
        let _ = std::fs::remove_file(&queue_state_path);

        let (queue, mut event_rx) = DownloadQueue::new(
            4, // max_concurrent_downloads
            2, // max_concurrent_tasks
            queue_state_path.clone(),
        );

        // 事件监听器
        let event_handle = tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    QueueEvent::TaskAdded { task_id } => {
                        println!("Task added: {}", task_id);
                    }
                    QueueEvent::TaskStarted { task_id } => {
                        println!("Task started: {}", task_id);
                    }
                    QueueEvent::TaskProgress {
                        task_id,
                        downloaded,
                        total,
                    } => {
                        let progress = (downloaded as f64 / total as f64) * 100.0;
                        println!("Task {} progress: {:.2}%", task_id, progress);
                    }
                    QueueEvent::TaskCompleted { task_id } => {
                        println!("Task completed: {}", task_id);
                    }
                    QueueEvent::TaskFailed { task_id, error } => {
                        println!("Task failed: {} - {}", task_id, error);
                    }
                    QueueEvent::TaskPaused { task_id } => {
                        println!("Task paused: {}", task_id);
                    }
                    QueueEvent::TaskResumed { task_id } => {
                        println!("Task resumed: {}", task_id);
                    }
                    QueueEvent::TaskCancelled { task_id } => {
                        println!("Task cancelled: {}", task_id);
                    }
                }
            }
        });

        // 添加测试任务
        let task1 = queue
            .add_task(
                "https://speed.hetzner.de/10MB.bin".to_string(),
                temp_dir.join("test_file1.bin"),
            )
            .await?;

        let task2 = queue
            .add_task(
                "https://speed.hetzner.de/10MB.bin".to_string(),
                temp_dir.join("test_file2.bin"),
            )
            .await?;

        println!("Added tasks: {} and {}", task1, task2);

        // 等待一段时间
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 测试暂停
        println!("Pausing task: {}", task1);
        queue.pause_task(&task1).await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 测试恢复
        println!("Resuming task: {}", task1);
        queue.resume_task(&task1).await?;

        // 获取所有任务
        let all_tasks = queue.get_all_tasks().await;
        println!("Total tasks: {}", all_tasks.len());
        for task in &all_tasks {
            println!(
                "Task {}: status={:?}, downloaded={}/{}",
                task.id, task.status, task.downloaded, task.total_size
            );
        }

        // 等待任务完成
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        // 清理
        let _ = std::fs::remove_file(temp_dir.join("test_file1.bin"));
        let _ = std::fs::remove_file(temp_dir.join("test_file2.bin"));
        let _ = std::fs::remove_file(&queue_state_path);

        event_handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_queue_persistence() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let queue_state_path = temp_dir.join("test_persistence_queue.json");

        // 清理之前的测试文件
        let _ = std::fs::remove_file(&queue_state_path);

        // 创建队列并添加任务
        {
            let (queue, _event_rx) = DownloadQueue::new(4, 2, queue_state_path.clone());

            let _task1 = queue
                .add_task(
                    "https://speed.hetzner.de/10MB.bin".to_string(),
                    temp_dir.join("persist_test1.bin"),
                )
                .await?;

            let _task2 = queue
                .add_task(
                    "https://speed.hetzner.de/10MB.bin".to_string(),
                    temp_dir.join("persist_test2.bin"),
                )
                .await?;

            // 等待状态保存
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        // 创建新队列并加载状态
        {
            let (queue, _event_rx) = DownloadQueue::new(4, 2, queue_state_path.clone());

            queue.load_from_state().await?;

            let tasks = queue.get_all_tasks().await;
            assert_eq!(tasks.len(), 2, "Should load 2 tasks from state");

            println!("Successfully loaded {} tasks from state", tasks.len());
        }

        // 清理
        let _ = std::fs::remove_file(&queue_state_path);
        let _ = std::fs::remove_file(temp_dir.join("persist_test1.bin"));
        let _ = std::fs::remove_file(temp_dir.join("persist_test2.bin"));

        Ok(())
    }

    #[tokio::test]
    async fn test_cancel_task() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let queue_state_path = temp_dir.join("test_cancel_queue.json");

        let _ = std::fs::remove_file(&queue_state_path);

        let (queue, mut event_rx) = DownloadQueue::new(4, 1, queue_state_path.clone());

        // 事件监听
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                if let QueueEvent::TaskCancelled { task_id } = event {
                    println!("Task cancelled: {}", task_id);
                }
            }
        });

        let task_id = queue
            .add_task(
                "https://speed.hetzner.de/100MB.bin".to_string(),
                temp_dir.join("cancel_test.bin"),
            )
            .await?;

        // 等待下载开始
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // 取消任务
        queue.cancel_task(&task_id).await?;

        // 验证任务状态
        if let Some(task) = queue.get_task(&task_id).await {
            assert_eq!(task.status, TaskStatus::Cancelled);
            println!("Task successfully cancelled");
        }

        // 清理
        let _ = std::fs::remove_file(&queue_state_path);

        Ok(())
    }
}
