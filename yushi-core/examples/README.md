# YuShi 示例程序

本目录包含了 YuShi 下载器的各种使用示例。

## 示例列表

### 1. `download_queue.rs` - 基础队列下载

展示下载队列的基本功能：
- 创建下载队列
- 添加多个下载任务
- 任务暂停和恢复
- 查看任务状态
- 清理已完成任务

**运行：**
```bash
cargo run --example download_queue
```

**功能演示：**
- ✅ 多任务并发下载
- ✅ 任务状态管理
- ✅ 暂停/恢复控制
- ✅ 进度追踪（带速度和 ETA）
- ✅ 事件监听

---

### 2. `advanced_download.rs` - 高级功能示例

展示 YuShi 的所有高级功能，包含三个子示例：

#### 示例 1: 自定义配置下载
- 自定义并发连接数
- 设置分块大小
- 速度限制
- 自定义 HTTP 头
- User-Agent 设置
- 代理支持（可选）

#### 示例 2: 优先级和文件校验
- 高/中/低优先级任务
- MD5 文件校验
- 自动重命名冲突文件
- 实时速度和 ETA 显示
- 校验事件监听

#### 示例 3: 完成回调
- 设置任务完成回调
- 成功/失败处理
- 后续操作示例

**运行：**
```bash
cargo run --example advanced_download
```

**功能演示：**
- ✅ 自定义下载配置
- ✅ 速度限制
- ✅ 任务优先级
- ✅ 文件校验（MD5/SHA256）
- ✅ 自动重命名
- ✅ 完成回调
- ✅ 自定义 HTTP 头
- ✅ 代理支持

---

## 运行前准备

### 1. 创建下载目录
```bash
mkdir -p downloads
```

### 2. 确保网络连接
示例使用 `https://speed.hetzner.de` 的测试文件，请确保可以访问。

### 3. 清理旧文件（可选）
```bash
rm -f downloads/*.bin
rm -f *_queue.json
```

---

## 示例输出

### 基础队列示例输出
```
=== 添加下载任务 ===
任务1 ID: abc12345-6789-...
任务2 ID: def67890-1234-...
任务3 ID: ghi34567-8901-...

✅ 任务已添加: abc12345
🚀 任务开始下载: abc12345
📊 任务 abc12345 进度: 15.30% (15728640/102760448) @ 5.23 MB/s (ETA: 16s)
📊 任务 abc12345 进度: 32.50% (33423360/102760448) @ 6.12 MB/s (ETA: 11s)
...
✨ 任务完成: abc12345

=== 暂停任务1 ===
⏸️  任务暂停: abc12345

=== 恢复任务1 ===
▶️  任务恢复: abc12345
```

### 高级功能示例输出
```
🚀 YuShi 高级下载示例

=== 示例 1: 自定义配置下载 ===
配置:
  - 并发连接: 8
  - 分块大小: 5 MB
  - 速度限制: 2 MB/s
开始下载，文件大小: 10.00 MB
✅ 下载完成!

=== 示例 2: 优先级和校验 ===
添加高优先级任务（带校验）...
➕ 添加任务: abc12345
添加普通优先级任务...
➕ 添加任务: def67890
添加低优先级任务...
➕ 添加任务: ghi34567

🚀 开始: abc12345
📊 abc12345: 45.2% @ 3.45 MB/s (ETA: 5s)
🔍 校验中: abc12345
✅ 校验通过: abc12345
✅ 完成: abc12345

=== 示例 3: 完成回调 ===
添加测试任务...
添加: abc12345
开始: abc12345
🎉 回调: 任务 abc12345 成功完成!
```

---

## 自定义示例

### 修改下载 URL
编辑示例文件，将 URL 改为你需要的：
```rust
let task = queue.add_task(
    "https://your-url.com/file.zip".to_string(),  // 修改这里
    PathBuf::from("downloads/file.zip"),
).await?;
```

### 修改速度限制
```rust
config.speed_limit = Some(5 * 1024 * 1024); // 5 MB/s
```

### 添加代理
```rust
config.proxy = Some("http://proxy.example.com:8080".to_string());
```

### 添加文件校验
```rust
// MD5 校验
Some(ChecksumType::Md5("your-md5-hash".to_string()))

// SHA256 校验
Some(ChecksumType::Sha256("your-sha256-hash".to_string()))
```

---

## 常见问题

### Q: 示例运行失败怎么办？
A: 检查：
1. 网络连接是否正常
2. downloads 目录是否存在
3. 磁盘空间是否充足
4. 防火墙是否允许连接

### Q: 如何计算文件的校验和？
A: 使用命令行工具：
```bash
# MD5
md5sum file.bin

# SHA256
sha256sum file.bin
```

### Q: 如何调试示例？
A: 添加日志输出：
```rust
println!("Debug: {:?}", some_variable);
```

### Q: 可以同时运行多个示例吗？
A: 可以，但建议使用不同的：
- 下载目录
- 队列状态文件名
- 避免端口冲突（如果使用代理）

---

## 进阶使用

### 1. 结合实际应用
```rust
// 从数据库读取下载列表
let urls = fetch_urls_from_database().await?;

for url in urls {
    queue.add_task(url, dest).await?;
}
```

### 2. 批量下载
```rust
let urls = vec![
    "https://example.com/file1.zip",
    "https://example.com/file2.zip",
    "https://example.com/file3.zip",
];

for (i, url) in urls.iter().enumerate() {
    queue.add_task(
        url.to_string(),
        PathBuf::from(format!("downloads/file{}.zip", i + 1)),
    ).await?;
}
```

### 3. 下载完成后处理
```rust
queue.set_on_complete(|task_id, result| async move {
    if result.is_ok() {
        // 解压文件
        unzip_file(&task_id).await;
        
        // 发送通知
        send_notification("Download completed").await;
        
        // 更新数据库
        update_database(&task_id).await;
    }
});
```

---

## 性能优化建议

1. **并发连接数**
   - 小文件（<10MB）：2-4 个
   - 中等文件（10-100MB）：4-6 个
   - 大文件（>100MB）：4-8 个

2. **同时下载任务数**
   - 低带宽：1-2 个
   - 中等带宽：2-4 个
   - 高带宽：4-8 个

3. **分块大小**
   - 快速网络：5-10 MB
   - 慢速网络：1-5 MB

4. **速度限制**
   - 根据实际带宽设置
   - 考虑其他应用的带宽需求

---

## 更多信息

- 完整 API 文档：查看 `../FEATURES.md`
- 模块结构：查看 `../MODULE_STRUCTURE.md`
- 源代码：查看 `../src/`

---

## 贡献

欢迎提交更多示例！如果你有好的使用案例，请：
1. 创建新的示例文件
2. 添加详细注释
3. 更新本 README
4. 提交 Pull Request
