use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 窗口状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 窗口 X 坐标
    pub x: i32,
    /// 窗口 Y 坐标
    pub y: i32,
    /// 是否最大化
    pub maximized: bool,
    /// 侧边栏是否展开
    pub sidebar_open: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            x: -1, // -1 表示居中
            y: -1,
            maximized: false,
            sidebar_open: true,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 默认下载路径
    pub default_download_path: PathBuf,
    /// 每个任务的最大并发下载连接数
    pub max_concurrent_downloads: usize,
    /// 队列中同时运行的最大任务数
    pub max_concurrent_tasks: usize,
    /// 分块大小（字节）
    pub chunk_size: u64,
    /// 连接超时（秒）
    pub timeout: u64,
    /// 用户代理
    pub user_agent: String,
    /// 主题设置 (light, dark, system)
    pub theme: String,
    /// 窗口状态
    #[serde(default)]
    pub window: WindowState,
}

impl Default for AppConfig {
    fn default() -> Self {
        // 获取用户下载目录
        let default_path = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        Self {
            default_download_path: default_path,
            max_concurrent_downloads: 4,
            max_concurrent_tasks: 3,
            chunk_size: 10 * 1024 * 1024, // 10MB
            timeout: 30,
            user_agent: "YuShi/0.1.0".to_string(),
            theme: "system".to_string(),
            window: WindowState::default(),
        }
    }
}

impl AppConfig {
    /// 从文件加载配置
    pub async fn load(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let content = fs_err::tokio::read_to_string(path).await?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // 如果配置文件不存在，返回默认配置
            Ok(Self::default())
        }
    }

    /// 保存配置到文件
    pub async fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs_err::tokio::write(path, content).await?;
        Ok(())
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        if self.max_concurrent_downloads == 0 {
            anyhow::bail!("max_concurrent_downloads must be greater than 0");
        }
        if self.max_concurrent_tasks == 0 {
            anyhow::bail!("max_concurrent_tasks must be greater than 0");
        }
        if self.chunk_size == 0 {
            anyhow::bail!("chunk_size must be greater than 0");
        }
        if self.timeout == 0 {
            anyhow::bail!("timeout must be greater than 0");
        }
        Ok(())
    }
}
