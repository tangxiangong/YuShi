use crate::{
    downloader::YuShi,
    state::QueueState,
    types::{DownloadTask, ProgressEvent, QueueEvent, TaskStatus},
};
use anyhow::{Result, anyhow};
use fs_err::tokio as fs;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::{
    sync::{RwLock, mpsc},
    task::JoinHandle,
};
use uuid::Uuid;

/// 下载队列管理器
pub struct DownloadQueue {
    yushi: Arc<YuShi>,
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    active_downloads: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    max_concurrent_tasks: usize,
    queue_state_path: PathBuf,
    event_tx: mpsc::Sender<QueueEvent>,
}

impl DownloadQueue {
    /// 创建新的下载队列
    ///
    /// # 参数
    /// * `max_concurrent_downloads` - 每个任务的最大并发下载连接数
    /// * `max_concurrent_tasks` - 队列中同时运行的最大任务数
    /// * `queue_state_path` - 队列状态持久化文件路径
    ///
    /// # 返回
    /// 返回队列实例和事件接收器
    pub fn new(
        max_concurrent_downloads: usize,
        max_concurrent_tasks: usize,
        queue_state_path: PathBuf,
    ) -> (Self, mpsc::Receiver<QueueEvent>) {
        let (event_tx, event_rx) = mpsc::channel(1024);

        let queue = Self {
            yushi: Arc::new(YuShi::new(max_concurrent_downloads)),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_tasks,
            queue_state_path,
            event_tx,
        };

        (queue, event_rx)
    }

    /// 从持久化状态加载队列
    pub async fn load_from_state(&self) -> Result<()> {
        if let Some(state) = QueueState::load(&self.queue_state_path).await? {
            let mut tasks = self.tasks.write().await;
            for task in state.tasks {
                tasks.insert(task.id.clone(), task);
            }
        }
        Ok(())
    }

    /// 保存队列状态
    async fn save_state(&self) -> Result<()> {
        let tasks = self.tasks.read().await;
        let task_list: Vec<DownloadTask> = tasks.values().cloned().collect();

        let state = QueueState { tasks: task_list };
        state.save(&self.queue_state_path).await?;
        Ok(())
    }

    /// 添加下载任务
    ///
    /// # 参数
    /// * `url` - 下载 URL
    /// * `dest` - 目标文件路径
    ///
    /// # 返回
    /// 返回任务 ID
    pub async fn add_task(&self, url: String, dest: PathBuf) -> Result<String> {
        let task_id = Uuid::new_v4().to_string();

        let task = DownloadTask {
            id: task_id.clone(),
            url,
            dest,
            status: TaskStatus::Pending,
            total_size: 0,
            downloaded: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            error: None,
        };

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }

        self.save_state().await?;
        let _ = self
            .event_tx
            .send(QueueEvent::TaskAdded {
                task_id: task_id.clone(),
            })
            .await;

        // 尝试启动任务
        self.process_queue().await?;

        Ok(task_id)
    }

    /// 处理队列，启动待处理的任务
    async fn process_queue(&self) -> Result<()> {
        let active_count = self.active_downloads.read().await.len();
        if active_count >= self.max_concurrent_tasks {
            return Ok(());
        }

        let pending_tasks: Vec<String> = {
            let tasks = self.tasks.read().await;
            tasks
                .values()
                .filter(|t| t.status == TaskStatus::Pending)
                .map(|t| t.id.clone())
                .collect()
        };

        for task_id in pending_tasks
            .iter()
            .take(self.max_concurrent_tasks - active_count)
        {
            self.start_task(task_id).await?;
        }

        Ok(())
    }

    /// 启动单个任务
    async fn start_task(&self, task_id: &str) -> Result<()> {
        let task = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow!("Task not found"))?;

            if task.status != TaskStatus::Pending && task.status != TaskStatus::Paused {
                return Ok(());
            }

            task.status = TaskStatus::Downloading;
            task.clone()
        };

        self.save_state().await?;
        let _ = self
            .event_tx
            .send(QueueEvent::TaskStarted {
                task_id: task_id.to_string(),
            })
            .await;

        let yushi = Arc::clone(&self.yushi);
        let tasks = Arc::clone(&self.tasks);
        let active_downloads = Arc::clone(&self.active_downloads);
        let queue_event_tx = self.event_tx.clone();
        let task_id_owned = task_id.to_string();
        let queue_state_path = self.queue_state_path.clone();

        let handle = tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel(1024);
            let task_id_clone = task_id_owned.clone();
            let queue_event_tx_clone = queue_event_tx.clone();
            let tasks_clone = Arc::clone(&tasks);

            // 进度监听器
            tokio::spawn(async move {
                let mut total = 0u64;
                let mut downloaded = 0u64;

                while let Some(event) = rx.recv().await {
                    match event {
                        ProgressEvent::Initialized { total_size } => {
                            total = total_size;
                            let mut tasks = tasks_clone.write().await;
                            if let Some(task) = tasks.get_mut(&task_id_clone) {
                                task.total_size = total_size;
                            }
                        }
                        ProgressEvent::ChunkUpdated { delta, .. } => {
                            downloaded += delta;
                            let mut tasks = tasks_clone.write().await;
                            if let Some(task) = tasks.get_mut(&task_id_clone) {
                                task.downloaded = downloaded;
                            }
                            let _ = queue_event_tx_clone
                                .send(QueueEvent::TaskProgress {
                                    task_id: task_id_clone.clone(),
                                    downloaded,
                                    total,
                                })
                                .await;
                        }
                        ProgressEvent::Finished => {}
                        ProgressEvent::Failed(_) => {}
                    }
                }
            });

            // 执行下载
            let result = yushi
                .download(&task.url, task.dest.to_str().unwrap(), tx)
                .await;

            // 更新任务状态
            let mut tasks = tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id_owned) {
                match result {
                    Ok(_) => {
                        task.status = TaskStatus::Completed;
                        let _ = queue_event_tx
                            .send(QueueEvent::TaskCompleted {
                                task_id: task_id_owned.clone(),
                            })
                            .await;
                    }
                    Err(e) => {
                        task.status = TaskStatus::Failed;
                        task.error = Some(e.to_string());
                        let _ = queue_event_tx
                            .send(QueueEvent::TaskFailed {
                                task_id: task_id_owned.clone(),
                                error: e.to_string(),
                            })
                            .await;
                    }
                }
            }

            // 保存状态
            let task_list: Vec<DownloadTask> = tasks.values().cloned().collect();
            let state = QueueState { tasks: task_list };
            if let Ok(data) = serde_json::to_string_pretty(&state) {
                let _ = fs::write(&queue_state_path, data).await;
            }

            // 从活动下载中移除
            active_downloads.write().await.remove(&task_id_owned);
        });

        self.active_downloads
            .write()
            .await
            .insert(task_id.to_string(), handle);

        Ok(())
    }

    /// 暂停任务
    pub async fn pause_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow!("Task not found"))?;

        if task.status == TaskStatus::Downloading {
            // 取消当前的下载任务
            let mut active = self.active_downloads.write().await;
            if let Some(handle) = active.remove(task_id) {
                handle.abort();
            }

            task.status = TaskStatus::Paused;
            drop(tasks);
            drop(active);

            self.save_state().await?;
            let _ = self
                .event_tx
                .send(QueueEvent::TaskPaused {
                    task_id: task_id.to_string(),
                })
                .await;
        }

        Ok(())
    }

    /// 恢复任务
    pub async fn resume_task(&self, task_id: &str) -> Result<()> {
        {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow!("Task not found"))?;

            if task.status == TaskStatus::Paused {
                task.status = TaskStatus::Pending;
                drop(tasks);

                self.save_state().await?;
                let _ = self
                    .event_tx
                    .send(QueueEvent::TaskResumed {
                        task_id: task_id.to_string(),
                    })
                    .await;
            }
        }

        self.process_queue().await?;
        Ok(())
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        // 如果正在下载，先停止
        let mut active = self.active_downloads.write().await;
        if let Some(handle) = active.remove(task_id) {
            handle.abort();
        }
        drop(active);

        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;

            // 删除下载文件和状态文件
            let _ = fs::remove_file(&task.dest).await;
            let state_path = task.dest.with_extension("json");
            let _ = fs::remove_file(state_path).await;
        }
        drop(tasks);

        self.save_state().await?;
        let _ = self
            .event_tx
            .send(QueueEvent::TaskCancelled {
                task_id: task_id.to_string(),
            })
            .await;

        // 处理队列中的下一个任务
        self.process_queue().await?;

        Ok(())
    }

    /// 移除已完成或已取消的任务
    pub async fn remove_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get(task_id)
            && (task.status == TaskStatus::Completed
                || task.status == TaskStatus::Cancelled
                || task.status == TaskStatus::Failed)
        {
            tasks.remove(task_id);
            drop(tasks);
            self.save_state().await?;
            return Ok(());
        }
        Err(anyhow!("Cannot remove task in current status"))
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 获取单个任务
    pub async fn get_task(&self, task_id: &str) -> Option<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 清空所有已完成的任务
    pub async fn clear_completed(&self) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.retain(|_, task| task.status != TaskStatus::Completed);
        drop(tasks);
        self.save_state().await?;
        Ok(())
    }
}
