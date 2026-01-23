use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 已完成的下载任务记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTask {
    /// 任务 ID
    pub id: String,
    /// 下载 URL
    pub url: String,
    /// 保存路径
    pub dest: PathBuf,
    /// 文件大小（字节）
    pub total_size: u64,
    /// 完成时间戳
    pub completed_at: u64,
    /// 下载耗时（秒）
    pub duration: u64,
    /// 平均速度（字节/秒）
    pub avg_speed: u64,
}

/// 下载历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadHistory {
    /// 已完成的任务列表
    pub completed_tasks: Vec<CompletedTask>,
    /// 最大历史记录数
    pub max_history: usize,
}

impl Default for DownloadHistory {
    fn default() -> Self {
        Self {
            completed_tasks: Vec::new(),
            max_history: 100,
        }
    }
}

impl DownloadHistory {
    /// 添加已完成的任务到历史记录
    pub fn add_completed(&mut self, task: CompletedTask) {
        // 添加到列表开头（最新的在前面）
        self.completed_tasks.insert(0, task);

        // 如果超过最大数量，删除最旧的记录
        if self.completed_tasks.len() > self.max_history {
            self.completed_tasks.truncate(self.max_history);
        }
    }

    /// 清除所有历史记录
    pub fn clear(&mut self) {
        self.completed_tasks.clear();
    }

    /// 删除指定的历史记录
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.completed_tasks.iter().position(|t| t.id == id) {
            self.completed_tasks.remove(pos);
            true
        } else {
            false
        }
    }

    /// 获取所有历史记录
    pub fn get_all(&self) -> &[CompletedTask] {
        &self.completed_tasks
    }

    /// 搜索历史记录
    pub fn search(&self, query: &str) -> Vec<CompletedTask> {
        let query_lower = query.to_lowercase();
        self.completed_tasks
            .iter()
            .filter(|task| {
                task.url.to_lowercase().contains(&query_lower)
                    || task
                        .dest
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// 从文件加载历史记录
    pub async fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs_err::tokio::read_to_string(path).await?;
            let history: DownloadHistory = serde_json::from_str(&content)?;
            Ok(history)
        } else {
            Ok(Self::default())
        }
    }

    /// 保存历史记录到文件
    pub async fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs_err::tokio::write(path, content).await?;
        Ok(())
    }
}
