use anyhow::Result;
use tokio::sync::mpsc;
use yushi_core::{DownloadConfig, DownloadMode, ProgressEvent, YuShi};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 演示流式下载（支持无 Content-Length 的服务器）");

    let (tx, mut rx) = mpsc::channel(1024);

    // 配置为流式下载模式
    let config = DownloadConfig {
        mode: DownloadMode::Streaming,
        ..Default::default()
    };

    let downloader = YuShi::with_config(config);

    // 进度监听器
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                ProgressEvent::Initialized { total_size } => {
                    if let Some(size) = total_size {
                        println!("📏 文件大小: {:.2} MB", size as f64 / 1024.0 / 1024.0);
                    } else {
                        println!("📡 流式下载开始（大小未知）");
                    }
                }
                ProgressEvent::StreamUpdated { downloaded } => {
                    println!("📊 已下载: {:.2} MB", downloaded as f64 / 1024.0 / 1024.0);
                }
                ProgressEvent::ChunkUpdated { delta, .. } => {
                    println!("📊 分块下载: +{:.2} KB", delta as f64 / 1024.0);
                }
                ProgressEvent::Finished => {
                    println!("✅ 下载完成!");
                    break;
                }
                ProgressEvent::Failed(e) => {
                    eprintln!("❌ 下载失败: {}", e);
                    break;
                }
            }
        }
    });

    let temp_dir = std::env::temp_dir();
    let dest_path = temp_dir.join("streaming_example.bin");

    // 清理之前的文件
    let _ = std::fs::remove_file(&dest_path);

    println!("📥 开始下载到: {}", dest_path.display());

    // 使用一个支持流式下载的 URL
    downloader
        .download(
            "https://httpbin.org/bytes/1048576", // 1MB 测试文件
            dest_path.to_str().unwrap(),
            tx,
        )
        .await?;

    // 验证下载结果
    if dest_path.exists() {
        let metadata = std::fs::metadata(&dest_path)?;
        println!(
            "📦 文件大小: {:.2} MB",
            metadata.len() as f64 / 1024.0 / 1024.0
        );

        // 清理
        let _ = std::fs::remove_file(&dest_path);
        println!("🧹 清理完成");
    }

    Ok(())
}
