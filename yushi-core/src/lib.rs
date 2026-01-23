//! YuShi - 高性能异步下载库
//!
//! 提供单文件下载和队列管理功能，支持断点续传、并发下载等特性。

pub mod downloader;
pub mod nbyte;
pub mod queue;
pub mod state;
pub mod types;
pub mod utils;

// 重新导出公共 API
pub use downloader::YuShi;
pub use queue::DownloadQueue;
pub use types::{
    ChecksumType, DownloadConfig, DownloadMode, DownloadTask, Priority, ProgressEvent, QueueEvent,
    TaskStatus,
};
pub use utils::{SpeedCalculator, auto_rename, verify_file};
