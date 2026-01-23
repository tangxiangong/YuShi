use crate::types::ChecksumType;
use anyhow::Result;
use fs_err::tokio as fs;
use md5::{Digest, Md5};
use sha2::Sha256;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio::io::AsyncReadExt;

/// Download Speed Limiter
#[derive(Debug, Clone)]
pub struct SpeedLimiter {
    limit: u64,
    last_check: Instant,
    bytes_in_period: u64,
}

impl SpeedLimiter {
    /// Create new speed limiter with limit in bytes per second
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            last_check: Instant::now(),
            bytes_in_period: 0,
        }
    }

    pub async fn wait(&mut self, bytes: u64) {
        let limit = self.limit;
        self.bytes_in_period += bytes;
        let elapsed = self.last_check.elapsed();

        if elapsed.as_secs() >= 1 {
            self.last_check = Instant::now();
            self.bytes_in_period = 0;
        } else if self.bytes_in_period > limit {
            let wait_time = Duration::from_secs(1) - elapsed;
            tokio::time::sleep(wait_time).await;
            self.last_check = Instant::now();
            self.bytes_in_period = 0;
        }
    }
}

/// Download Speed Calculator
#[derive(Debug, Clone)]
pub struct SpeedCalculator {
    start_time: Instant,
    last_update: Instant,
    last_bytes: u64,
    current_speed: u64,
}

impl SpeedCalculator {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_update: now,
            last_bytes: 0,
            current_speed: 0,
        }
    }

    /// 更新速度统计
    pub fn update(&mut self, total_downloaded: u64) -> u64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        if elapsed >= 1.0 {
            let bytes_diff = total_downloaded.saturating_sub(self.last_bytes);
            self.current_speed = (bytes_diff as f64 / elapsed) as u64;
            self.last_update = now;
            self.last_bytes = total_downloaded;
        }

        self.current_speed
    }

    /// 计算 ETA（预计剩余时间，秒）
    pub fn calculate_eta(&self, downloaded: u64, total: u64) -> Option<u64> {
        if self.current_speed == 0 || downloaded >= total {
            return None;
        }

        let remaining = total - downloaded;
        Some(remaining / self.current_speed)
    }

    /// 获取平均速度
    pub fn average_speed(&self, total_downloaded: u64) -> u64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (total_downloaded as f64 / elapsed) as u64
        } else {
            0
        }
    }
}

impl Default for SpeedCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// 文件校验
pub async fn verify_file(path: &Path, checksum: &ChecksumType) -> Result<bool> {
    let mut file = fs::File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let result = match checksum {
        ChecksumType::Md5(expected) => {
            let mut hasher = Md5::new();
            hasher.update(&buffer);
            let hash = hex::encode(hasher.finalize());
            hash.eq_ignore_ascii_case(expected)
        }
        ChecksumType::Sha256(expected) => {
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            let hash = hex::encode(hasher.finalize());
            hash.eq_ignore_ascii_case(expected)
        }
    };

    Ok(result)
}

/// 自动重命名文件以避免冲突
pub fn auto_rename(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    let mut counter = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, ext)
        };

        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct XByte {
    pub(crate) quotient: u64,
    pub(crate) remainder: u64,
    pub(crate) unit: Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    B,
    KB,
    MB,
    GB,
    TB,
    PB,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Unit::B => "B",
                Unit::KB => "KB",
                Unit::MB => "MB",
                Unit::GB => "GB",
                Unit::TB => "TB",
                Unit::PB => "PB",
            }
        )
    }
}

impl XByte {
    const SHIFT_KB: u64 = 10;
    const SHIFT_MB: u64 = 20;
    const SHIFT_GB: u64 = 30;
    const SHIFT_TB: u64 = 40;
    const SHIFT_PB: u64 = 50;

    const SCALE_KB: f64 = 1.0 / (1u64 << Self::SHIFT_KB) as f64;
    const SCALE_MB: f64 = 1.0 / (1u64 << Self::SHIFT_MB) as f64;
    const SCALE_GB: f64 = 1.0 / (1u64 << Self::SHIFT_GB) as f64;
    const SCALE_TB: f64 = 1.0 / (1u64 << Self::SHIFT_TB) as f64;
    const SCALE_PB: f64 = 1.0 / (1u64 << Self::SHIFT_PB) as f64;

    pub fn new(quotient: u64, remainder: u64, unit: Unit) -> Self {
        Self {
            quotient,
            remainder,
            unit,
        }
    }

    pub fn from_bytes(bytes: u64) -> Self {
        if bytes >= (1 << Self::SHIFT_PB) {
            let q = bytes >> Self::SHIFT_PB;
            let r = bytes & ((1 << Self::SHIFT_PB) - 1);
            XByte::new(q, r, Unit::PB)
        } else if bytes >= (1 << Self::SHIFT_TB) {
            let q = bytes >> Self::SHIFT_TB;
            let r = bytes & ((1 << Self::SHIFT_TB) - 1);
            XByte::new(q, r, Unit::TB)
        } else if bytes >= (1 << Self::SHIFT_GB) {
            let q = bytes >> Self::SHIFT_GB;
            let r = bytes & ((1 << Self::SHIFT_GB) - 1);
            XByte::new(q, r, Unit::GB)
        } else if bytes >= (1 << Self::SHIFT_MB) {
            let q = bytes >> Self::SHIFT_MB;
            let r = bytes & ((1 << Self::SHIFT_MB) - 1);
            XByte::new(q, r, Unit::MB)
        } else if bytes >= (1 << Self::SHIFT_KB) {
            let q = bytes >> Self::SHIFT_KB;
            let r = bytes & ((1 << Self::SHIFT_KB) - 1);
            XByte::new(q, r, Unit::KB)
        } else {
            XByte::new(bytes, 0, Unit::B)
        }
    }

    pub fn to_bytes(&self) -> u64 {
        match self.unit {
            Unit::B => self.quotient,
            Unit::KB => (self.quotient << Self::SHIFT_KB) | self.remainder,
            Unit::MB => (self.quotient << Self::SHIFT_MB) | self.remainder,
            Unit::GB => (self.quotient << Self::SHIFT_GB) | self.remainder,
            Unit::TB => (self.quotient << Self::SHIFT_TB) | self.remainder,
            Unit::PB => (self.quotient << Self::SHIFT_PB) | self.remainder,
        }
    }

    pub fn to_float(&self) -> f64 {
        match self.unit {
            Unit::B => self.quotient as f64,
            Unit::KB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_KB),
            Unit::MB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_MB),
            Unit::GB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_GB),
            Unit::TB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_TB),
            Unit::PB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_PB),
        }
    }

    pub fn quotient(&self) -> u64 {
        self.quotient
    }

    pub fn remainder(&self) -> u64 {
        self.remainder
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }
}

impl std::ops::Add<XByte> for XByte {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let total_bytes = self.to_bytes() + other.to_bytes();
        XByte::from_bytes(total_bytes)
    }
}

impl std::ops::Add<&XByte> for XByte {
    type Output = Self;

    fn add(self, other: &Self) -> Self {
        let total_bytes = self.to_bytes() + other.to_bytes();
        XByte::from_bytes(total_bytes)
    }
}

impl std::ops::Add<XByte> for &XByte {
    type Output = XByte;

    fn add(self, other: XByte) -> Self::Output {
        let total_bytes = self.to_bytes() + other.to_bytes();
        XByte::from_bytes(total_bytes)
    }
}

impl std::ops::Add<&XByte> for &XByte {
    type Output = XByte;

    fn add(self, other: &XByte) -> Self::Output {
        let total_bytes = self.to_bytes() + other.to_bytes();
        XByte::from_bytes(total_bytes)
    }
}

impl std::fmt::Display for XByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.to_float();
        write!(f, "{:.2} {}", value, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_rename() {
        let path = Path::new("/tmp/test.txt");
        let renamed = auto_rename(path);
        // 如果文件不存在，应该返回原路径
        assert_eq!(renamed, path);
    }

    #[tokio::test]
    async fn test_speed_calculator() {
        let mut calc = SpeedCalculator::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 模拟下载了 1MB
        let speed = calc.update(1024 * 1024);
        // 速度应该大于 0
        assert!(speed > 0);
    }
}
