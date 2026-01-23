use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use yushi_core::nbyte::Storage;

pub struct ProgressManager {
    multi: MultiProgress,
    bars: Arc<RwLock<HashMap<String, ProgressBar>>>,
}

impl ProgressManager {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            bars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, task_id: String, total_size: Option<u64>) {
        let pb = if let Some(size) = total_size {
            // 已知大小，使用进度条
            let pb = self.multi.add(ProgressBar::new(size));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb
        } else {
            // 未知大小，使用旋转器
            let pb = self.multi.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec}) - 流式下载")
                    .unwrap(),
            );
            pb
        };
        pb.set_message(format!("📥 {}", &task_id[..8]));

        let mut bars = self.bars.write().await;
        bars.insert(task_id, pb);
    }

    pub async fn update_progress(&self, task_id: &str, downloaded: u64, speed: u64) {
        let bars = self.bars.read().await;
        if let Some(pb) = bars.get(task_id) {
            pb.set_position(downloaded);
            let speed_mb = speed as f64 / 1024.0 / 1024.0;
            pb.set_message(format!("📥 {} @ {:.2} MB/s", &task_id[..8], speed_mb));
        }
    }

    pub async fn finish_task(&self, task_id: &str, success: bool) {
        let mut bars = self.bars.write().await;
        if let Some(pb) = bars.remove(task_id) {
            if success {
                pb.finish_with_message(format!("✅ {} 完成", &task_id[..8]));
            } else {
                pb.finish_with_message(format!("❌ {} 失败", &task_id[..8]));
            }
        }
    }
}

pub fn parse_speed_limit(limit: &str) -> Option<u64> {
    let limit = limit.trim().to_uppercase();
    let (num_str, unit) = if limit.ends_with('K') {
        (&limit[..limit.len() - 1], 1024u64)
    } else if limit.ends_with('M') {
        (&limit[..limit.len() - 1], 1024u64 * 1024)
    } else if limit.ends_with('G') {
        (&limit[..limit.len() - 1], 1024u64 * 1024 * 1024)
    } else {
        (limit.as_str(), 1u64)
    };

    num_str.parse::<u64>().ok().map(|n| n * unit)
}

pub fn format_size(bytes: u64) -> String {
    Storage::from_bytes(bytes).to_string()
}

pub fn print_success(msg: &str) {
    println!("{} {}", style("✓").green().bold(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", style("✗").red().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", style("ℹ").blue().bold(), msg);
}

#[allow(dead_code)]
pub fn print_warning(msg: &str) {
    println!("{} {}", style("⚠").yellow().bold(), msg);
}
