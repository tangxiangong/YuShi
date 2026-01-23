# 驭时

## 概述

驭时 (YuShi) 是一个功能强大的 Rust
异步下载库，支持单文件下载和队列管理，具有丰富的高级特性。

## ✨ 核心功能

### 1. 基础下载功能

#### 单文件下载器 (`YuShi`)

- ✅ **并发分块下载** - 支持多连接同时下载不同分块
- ✅ **流式下载** - 支持无 Content-Length 的服务器
- ✅ **自动模式选择** - 智能选择分块或流式下载
- ✅ **断点续传** - 自动保存和恢复下载进度
- ✅ **自动重试** - 失败时自动重试（最多 5 次）
- ✅ **进度追踪** - 实时报告下载进度

#### 下载队列 (`DownloadQueue`)

- ✅ **多任务管理** - 同时管理多个下载任务
- ✅ **并发控制** - 限制同时运行的任务数量
- ✅ **状态持久化** - 队列状态自动保存到文件
- ✅ **任务控制** - 暂停、恢复、取消任务

---

## 🚀 高级功能

### 2. 速度管理

#### 速度限制

```rust
let mut config = DownloadConfig::default();
config.speed_limit = Some(1024 * 1024); // 限速 1 MB/s
let downloader = YuShi::with_config(config);
```

**特性：**

- 全局速度限制
- 所有分块共享带宽限制
- 平滑的速度控制

#### 速度统计

```rust
// 自动计算的指标
task.speed       // 当前速度（字节/秒）
task.eta         // 预计剩余时间（秒）
```

**提供的信息：**

- 实时下载速度
- 平均下载速度
- 预计完成时间（ETA）

---

### 3. 网络配置

#### 自定义 HTTP 头

```rust
let mut config = DownloadConfig::default();
config.headers.insert("Cookie".to_string(), "session=xxx".to_string());
config.headers.insert("Referer".to_string(), "https://example.com".to_string());
config.user_agent = Some("MyApp/1.0".to_string());
```

**支持：**

- 自定义 User-Agent
- Cookie 支持
- Referer 和其他自定义头
- 认证头（Authorization）

#### 代理支持

```rust
let mut config = DownloadConfig::default();
config.proxy = Some("http://proxy.example.com:8080".to_string());
// 或 SOCKS5
config.proxy = Some("socks5://proxy.example.com:1080".to_string());
```

**支持的代理类型：**

- HTTP 代理
- HTTPS 代理
- SOCKS5 代理

#### 超时配置

```rust
let mut config = DownloadConfig::default();
config.timeout = 60; // 60 秒超时
```

---

### 4. 文件校验

#### MD5 校验

```rust
use yushi_core::{ChecksumType, Priority};

queue.add_task_with_options(
    url,
    dest,
    Priority::Normal,
    Some(ChecksumType::Md5("5d41402abc4b2a76b9719d911017c592".to_string())),
    false,
).await?;
```

#### SHA256 校验

```rust
queue.add_task_with_options(
    url,
    dest,
    Priority::Normal,
    Some(ChecksumType::Sha256("2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae".to_string())),
    false,
).await?;
```

**特性：**

- 下载完成后自动校验
- 校验失败自动标记为失败
- 支持 MD5 和 SHA256
- 校验事件通知

---

### 5. 任务优先级

#### 优先级类型

```rust
pub enum Priority {
    Low = 0,      // 低优先级
    Normal = 1,   // 普通优先级（默认）
    High = 2,     // 高优先级
}
```

#### 使用示例

```rust
// 高优先级任务
queue.add_task_with_options(
    url,
    dest,
    Priority::High,
    None,
    false,
).await?;
```

**行为：**

- 队列按优先级排序
- 高优先级任务优先执行
- 同优先级按添加顺序执行

---

### 6. 文件管理

#### 自动重命名

```rust
// 如果文件已存在，自动重命名
queue.add_task_with_options(
    url,
    PathBuf::from("file.zip"),
    Priority::Normal,
    None,
    true,  // 启用自动重命名
).await?;

// 结果：file.zip, file (1).zip, file (2).zip, ...
```

**特性：**

- 自动检测文件冲突
- 智能重命名（保留扩展名）
- 递增编号

#### 手动重命名工具

```rust
use yushi_core::auto_rename;

let new_path = auto_rename(Path::new("existing_file.txt"));
// 返回: existing_file (1).txt
```

---

### 7. 事件系统

#### 队列事件

```rust
pub enum QueueEvent {
    TaskAdded { task_id },
    TaskStarted { task_id },
    TaskProgress { task_id, downloaded, total, speed, eta },
    TaskCompleted { task_id },
    TaskFailed { task_id, error },
    TaskPaused { task_id },
    TaskResumed { task_id },
    TaskCancelled { task_id },
    VerifyStarted { task_id },
    VerifyCompleted { task_id, success },
}
```

#### 事件监听

```rust
let (queue, mut event_rx) = DownloadQueue::new(4, 2, state_path);

tokio::spawn(async move {
    while let Some(event) = event_rx.recv().await {
        match event {
            QueueEvent::TaskProgress { task_id, downloaded, total, speed, eta } => {
                let progress = (downloaded as f64 / total as f64) * 100.0;
                let speed_mb = speed as f64 / 1024.0 / 1024.0;
                println!("Task {}: {:.2}% ({:.2} MB/s, ETA: {:?}s)", 
                    task_id, progress, speed_mb, eta);
            }
            QueueEvent::VerifyCompleted { task_id, success } => {
                println!("Task {} verification: {}", task_id, 
                    if success { "passed" } else { "failed" });
            }
            // ... 处理其他事件
            _ => {}
        }
    }
});
```

---

### 8. 回调系统

#### 设置完成回调

```rust
let (mut queue, event_rx) = DownloadQueue::new(4, 2, state_path);

queue.set_on_complete(|task_id, result| async move {
    match result {
        Ok(_) => {
            println!("✅ Task {} completed successfully!", task_id);
            // 可以在这里执行后续操作
            // - 发送通知
            // - 解压文件
            // - 移动文件
            // - 更新数据库
        }
        Err(error) => {
            eprintln!("❌ Task {} failed: {}", task_id, error);
            // 错误处理
            // - 记录日志
            // - 发送警报
            // - 重试逻辑
        }
    }
});
```

**用途：**

- 下载完成后的自动化处理
- 文件后处理（解压、移动等）
- 通知发送
- 日志记录
- 数据库更新

---

## 📊 完整使用示例

### 高级下载配置

```rust
use yushi_core::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 创建自定义配置
    let mut config = DownloadConfig {
        max_concurrent: 8,                    // 8 个并发连接
        chunk_size: 5 * 1024 * 1024,         // 5MB 分块
        speed_limit: Some(2 * 1024 * 1024),  // 限速 2 MB/s
        headers: {
            let mut h = std::collections::HashMap::new();
            h.insert("Cookie".to_string(), "session=abc123".to_string());
            h
        },
        proxy: Some("http://proxy.example.com:8080".to_string()),
        timeout: 60,
        user_agent: Some("MyDownloader/1.0".to_string()),
    };

    // 2. 创建队列
    let (mut queue, mut event_rx) = DownloadQueue::new(4, 2, PathBuf::from("queue.json"));

    // 3. 设置完成回调
    queue.set_on_complete(|task_id, result| async move {
        match result {
            Ok(_) => println!("✅ {} completed!", task_id),
            Err(e) => eprintln!("❌ {} failed: {}", task_id, e),
        }
    });

    // 4. 启动事件监听
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                QueueEvent::TaskProgress { task_id, downloaded, total, speed, eta } => {
                    let progress = (downloaded as f64 / total as f64) * 100.0;
                    let speed_mb = speed as f64 / 1024.0 / 1024.0;
                    print!("\r{}: {:.1}% @ {:.2} MB/s", task_id, progress, speed_mb);
                    if let Some(eta_secs) = eta {
                        print!(" (ETA: {}s)", eta_secs);
                    }
                }
                QueueEvent::VerifyCompleted { task_id, success } => {
                    println!("\n{} verification: {}", task_id, 
                        if success { "✓" } else { "✗" });
                }
                _ => {}
            }
        }
    });

    // 5. 添加高优先级任务（带校验）
    queue.add_task_with_options(
        "https://example.com/important.zip".to_string(),
        PathBuf::from("downloads/important.zip"),
        Priority::High,
        Some(ChecksumType::Sha256("abc123...".to_string())),
        true,  // 自动重命名
    ).await?;

    // 6. 添加普通任务
    queue.add_task(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("downloads/file.zip"),
    ).await?;

    // 7. 等待完成
    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;

    Ok(())
}
```

---

## 🛠️ 工具函数

### 文件校验

```rust
use yushi_core::{verify_file, ChecksumType};

let is_valid = verify_file(
    Path::new("file.zip"),
    &ChecksumType::Md5("5d41402abc4b2a76b9719d911017c592".to_string()),
).await?;
```

### 自动重命名

```rust
use yushi_core::auto_rename;

let new_path = auto_rename(Path::new("existing.txt"));
```

### 速度计算

```rust
use yushi_core::SpeedCalculator;

let mut calc = SpeedCalculator::new();
let speed = calc.update(downloaded_bytes);
let eta = calc.calculate_eta(downloaded, total);
let avg_speed = calc.average_speed(downloaded);
```

---

## 📦 数据结构

### DownloadTask

```rust
pub struct DownloadTask {
    pub id: String,                    // 任务 ID
    pub url: String,                   // 下载 URL
    pub dest: PathBuf,                 // 目标路径
    pub status: TaskStatus,            // 状态
    pub total_size: u64,               // 总大小
    pub downloaded: u64,               // 已下载
    pub created_at: u64,               // 创建时间
    pub error: Option<String>,         // 错误信息
    pub priority: Priority,            // 优先级
    pub speed: u64,                    // 当前速度
    pub eta: Option<u64>,              // ETA
    pub headers: HashMap<String, String>, // 自定义头
    pub checksum: Option<ChecksumType>, // 校验
}
```

### DownloadConfig

```rust
pub struct DownloadConfig {
    pub max_concurrent: usize,         // 最大并发数
    pub chunk_size: u64,               // 分块大小
    pub speed_limit: Option<u64>,      // 速度限制
    pub headers: HashMap<String, String>, // HTTP 头
    pub proxy: Option<String>,         // 代理
    pub timeout: u64,                  // 超时
    pub user_agent: Option<String>,    // User-Agent
}
```

---

## 🎯 最佳实践

### 1. 速度限制

- 小文件：不限速或高限速
- 大文件：根据网络情况设置合理限速
- 多任务：考虑总带宽分配

### 2. 并发配置

- 小文件（<10MB）：2-4 个连接
- 中等文件（10-100MB）：4-6 个连接
- 大文件（>100MB）：4-8 个连接

### 3. 任务优先级

- 重要文件：High
- 普通文件：Normal
- 后台任务：Low

### 4. 文件校验

- 关键文件：必须校验
- 普通文件：可选校验
- 临时文件：不需要校验

### 5. 错误处理

- 使用事件系统监控错误
- 设置回调处理失败任务
- 记录详细日志

---

## 📈 性能特性

- ✅ **零拷贝** - 直接写入文件，无额外内存拷贝
- ✅ **异步 I/O** - 基于 Tokio 的高性能异步运行时
- ✅ **并发下载** - 充分利用多核和网络带宽
- ✅ **内存高效** - 流式处理，内存占用低
- ✅ **状态持久化** - 崩溃后可恢复

---

## 🔒 安全特性

- ✅ **文件校验** - MD5/SHA256 完整性验证
- ✅ **原子操作** - 状态更新使用锁保护
- ✅ **错误恢复** - 自动重试和断点续传
- ✅ **资源清理** - 取消任务时清理临时文件

---

## 📝 总结

YuShi 下载器提供了一整套完善的下载解决方案：

✅ **8 大核心功能**

1. 速度限制与统计
2. 自定义 HTTP 头
3. 代理支持
4. 文件校验（MD5/SHA256）
5. 优先级管理
6. 自动重命名
7. 事件系统
8. 完成回调

✅ **生产就绪**

- 完整的错误处理
- 状态持久化
- 断点续传
- 自动重试

✅ **易于使用**

- 清晰的 API
- 丰富的文档
- 完整的示例

✅ **高性能**

- 异步架构
- 并发下载
- 内存高效
