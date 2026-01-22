use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 任务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待开始
    Pending,
    /// 正在下载
    Downloading,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 下载任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    /// 任务唯一标识符
    pub id: String,
    /// 下载 URL
    pub url: String,
    /// 目标文件路径
    pub dest: PathBuf,
    /// 当前状态
    pub status: TaskStatus,
    /// 文件总大小（字节）
    pub total_size: u64,
    /// 已下载大小（字节）
    pub downloaded: u64,
    /// 创建时间戳（Unix 时间）
    pub created_at: u64,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 队列事件
#[derive(Debug, Clone)]
pub enum QueueEvent {
    /// 任务已添加
    TaskAdded { task_id: String },
    /// 任务开始下载
    TaskStarted { task_id: String },
    /// 任务进度更新
    TaskProgress {
        task_id: String,
        downloaded: u64,
        total: u64,
    },
    /// 任务完成
    TaskCompleted { task_id: String },
    /// 任务失败
    TaskFailed { task_id: String, error: String },
    /// 任务暂停
    TaskPaused { task_id: String },
    /// 任务恢复
    TaskResumed { task_id: String },
    /// 任务取消
    TaskCancelled { task_id: String },
}

/// 单文件下载进度事件
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// 初始化完成，获取到文件总大小
    Initialized { total_size: u64 },
    /// 分块下载进度更新
    ChunkUpdated { chunk_index: usize, delta: u64 },
    /// 下载完成
    Finished,
    /// 下载失败
    Failed(String),
}
