use crate::utils::{Unit, XByte};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// 下载完成回调类型
pub type DownloadCallback = Arc<
    dyn Fn(
            String,
            Result<(), String>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    /// 低优先级
    Low = 0,
    #[default]
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
}

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

/// 文件校验类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumType {
    /// MD5 校验
    Md5(String),
    /// SHA256 校验
    Sha256(String),
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
    /// 任务优先级
    #[serde(default)]
    pub priority: Priority,
    /// 当前下载速度（字节/秒）
    #[serde(default)]
    pub speed: u64,
    /// 预计剩余时间（秒）
    #[serde(default)]
    pub eta: Option<u64>,
    /// 自定义 HTTP 头
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 文件校验
    #[serde(default)]
    pub checksum: Option<ChecksumType>,
}

/// 队列事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
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
        speed: u64,
        eta: Option<u64>,
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
    /// 校验开始
    VerifyStarted { task_id: String },
    /// 校验完成
    VerifyCompleted { task_id: String, success: bool },
}

/// 单文件下载进度事件
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// 初始化完成，获取到文件总大小（None 表示未知大小）
    Initialized { total_size: Option<u64> },
    /// 分块下载进度更新
    ChunkDownloading { chunk_index: usize, delta: u64 },
    /// 流式下载进度更新
    StreamDownloading { downloaded: u64 },
    /// 下载完成
    Finished,
    /// 下载失败
    Failed(String),
}

/// 下载配置
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// 最大并发连接数
    pub max_concurrent: usize,
    /// 分块大小（字节）
    pub chunk_size: u64,
    /// 速度限制（字节/秒），None 表示不限速
    pub speed_limit: Option<u64>,
    /// 自定义 HTTP 头
    pub headers: HashMap<String, String>,
    /// 代理 URL
    pub proxy: Option<String>,
    /// 连接超时（秒）
    pub timeout: u64,
    /// 用户代理
    pub user_agent: Option<String>,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            chunk_size: XByte::new(10, 0, Unit::MB).to_bytes(),
            speed_limit: None,
            headers: HashMap::new(),
            proxy: None,
            timeout: 30,
            user_agent: Some("YuShi/1.0".to_string()),
        }
    }
}
