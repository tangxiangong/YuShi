use crate::{
    cli::DownloadArgs,
    ui::{format_size, parse_speed_limit, print_error, print_info, print_success},
};
use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tokio::sync::mpsc;
use yushi_core::{ChecksumType, DownloadConfig, ProgressEvent, YuShi};

pub async fn execute(args: DownloadArgs) -> Result<()> {
    // 确定输出路径
    let output = if let Some(path) = args.output {
        path
    } else {
        // 从 URL 提取文件名
        let filename = args
            .url
            .split('/')
            .next_back()
            .and_then(|s| s.split('?').next())
            .unwrap_or("download");
        PathBuf::from(filename)
    };

    print_info(&format!("下载: {}", args.url));
    print_info(&format!("保存到: {}", output.display()));

    // 构建配置
    let mut config = DownloadConfig {
        max_concurrent: args.connections,
        ..Default::default()
    };

    if let Some(limit_str) = &args.speed_limit {
        config.speed_limit = parse_speed_limit(limit_str);
        if let Some(limit) = config.speed_limit {
            print_info(&format!("速度限制: {}/s", format_size(limit)));
        }
    }

    if let Some(ua) = &args.user_agent {
        config.user_agent = Some(ua.clone());
    }

    if let Some(proxy) = &args.proxy {
        config.proxy = Some(proxy.clone());
        print_info(&format!("使用代理: {}", proxy));
    }

    // 解析自定义头
    for header in &args.header {
        if let Some((key, value)) = header.split_once(':') {
            config
                .headers
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // 创建下载器
    let downloader = YuShi::with_config(config);
    let (tx, mut rx) = mpsc::channel(1024);

    // 进度显示
    let quiet = args.quiet;
    let progress_handle = tokio::spawn(async move {
        let mut pb: Option<ProgressBar> = None;
        let mut downloaded = 0u64;

        while let Some(event) = rx.recv().await {
            match event {
                ProgressEvent::Initialized { total_size } => {
                    if !quiet {
                        if let Some(size) = total_size {
                            // 分块下载，已知文件大小
                            let bar = ProgressBar::new(size);
                            bar.set_style(
                                ProgressStyle::default_bar()
                                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                                    .unwrap()
                                    .progress_chars("#>-"),
                            );
                            pb = Some(bar);
                        } else {
                            // 流式下载，未知文件大小
                            let bar = ProgressBar::new_spinner();
                            bar.set_style(
                                ProgressStyle::default_spinner()
                                    .template("{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec}) - 流式下载")
                                    .unwrap(),
                            );
                            pb = Some(bar);
                        }
                    }
                }
                ProgressEvent::ChunkDownloading { delta, .. } => {
                    downloaded += delta;
                    if let Some(ref bar) = pb {
                        bar.set_position(downloaded);
                    }
                }
                ProgressEvent::StreamDownloading {
                    downloaded: stream_downloaded,
                } => {
                    downloaded = stream_downloaded;
                    if let Some(ref bar) = pb {
                        bar.set_position(downloaded);
                    }
                }
                ProgressEvent::Finished => {
                    if let Some(bar) = pb.take() {
                        bar.finish_with_message("下载完成");
                    }
                }
                ProgressEvent::Failed(e) => {
                    if let Some(bar) = pb.take() {
                        bar.finish_with_message(format!("下载失败: {}", e));
                    }
                }
            }
        }
    });

    // 执行下载
    let result = downloader
        .download(&args.url, output.to_str().unwrap(), tx)
        .await;

    progress_handle.await?;

    match result {
        Ok(_) => {
            // 文件校验
            if let Some(md5) = args.md5 {
                print_info("验证 MD5...");
                let checksum = ChecksumType::Md5(md5);
                match yushi_core::verify_file(&output, &checksum).await {
                    Ok(true) => print_success("MD5 校验通过"),
                    Ok(false) => {
                        print_error("MD5 校验失败");
                        return Err(anyhow!("MD5 校验失败"));
                    }
                    Err(e) => {
                        print_error(&format!("MD5 校验错误: {}", e));
                        return Err(e);
                    }
                }
            }

            if let Some(sha256) = args.sha256 {
                print_info("验证 SHA256...");
                let checksum = ChecksumType::Sha256(sha256);
                match yushi_core::verify_file(&output, &checksum).await {
                    Ok(true) => print_success("SHA256 校验通过"),
                    Ok(false) => {
                        print_error("SHA256 校验失败");
                        return Err(anyhow!("SHA256 校验失败"));
                    }
                    Err(e) => {
                        print_error(&format!("SHA256 校验错误: {}", e));
                        return Err(e);
                    }
                }
            }

            print_success(&format!("文件已保存到: {}", output.display()));
            Ok(())
        }
        Err(e) => {
            print_error(&format!("下载失败: {}", e));
            Err(e)
        }
    }
}
