use crate::types::DownloadTask;
use anyhow::Result;
use fs_err::tokio as fs;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 分块下载状态
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ChunkState {
    pub index: usize,
    pub start: u64,
    pub end: u64,
    pub current: u64,
    pub is_finished: bool,
}

/// 单文件下载状态
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DownloadState {
    pub url: String,
    /// 文件总大小，None 表示未知（流式下载）
    pub total_size: Option<u64>,
    pub chunks: Vec<ChunkState>,
    /// 是否为流式下载模式
    pub is_streaming: bool,
}

impl DownloadState {
    /// 保存状态到文件
    pub async fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string(self)?;
        fs::write(path, data).await?;
        Ok(())
    }

    /// 从文件加载状态
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path).await?;
        let state = serde_json::from_str(&content)?;
        Ok(Some(state))
    }
}

/// 队列状态
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct QueueState {
    pub tasks: Vec<DownloadTask>,
}

impl QueueState {
    /// 保存队列状态到文件
    pub async fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data).await?;
        Ok(())
    }

    /// 从文件加载队列状态
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path).await?;
        let state = serde_json::from_str(&content)?;
        Ok(Some(state))
    }
}
