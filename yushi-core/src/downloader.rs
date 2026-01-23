use crate::{
    state::{ChunkState, DownloadState},
    types::{DownloadConfig, ProgressEvent},
    utils::SpeedLimiter,
};
use anyhow::{Result, anyhow};
use fs_err::tokio as fs;
use futures::StreamExt;
use reqwest::{
    Client, Proxy,
    header::{CONTENT_LENGTH, RANGE, USER_AGENT},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncSeekExt, AsyncWriteExt, SeekFrom},
    sync::{RwLock, Semaphore, mpsc},
};

/// 单文件下载器
#[derive(Debug, Clone)]
pub struct YuShi {
    client: Client,
    config: DownloadConfig,
}

impl YuShi {
    /// 创建新的下载器实例
    ///
    /// # 参数
    /// * `max_concurrent` - 最大并发连接数（分块下载）
    pub fn new(max_concurrent: usize) -> Self {
        let config = DownloadConfig {
            max_concurrent,
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// 使用自定义配置创建下载器
    pub fn with_config(config: DownloadConfig) -> Self {
        let mut builder = Client::builder()
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(config.timeout));

        // 设置代理
        if let Some(proxy_url) = &config.proxy
            && let Ok(proxy) = Proxy::all(proxy_url)
        {
            builder = builder.proxy(proxy);
        }

        let client = builder.build().unwrap();

        Self { client, config }
    }

    /// 下载文件
    ///
    /// # 参数
    /// * `url` - 下载 URL
    /// * `dest` - 目标文件路径
    /// * `event_tx` - 进度事件发送器
    pub async fn download(
        &self,
        url: &str,
        dest: &str,
        event_tx: mpsc::Sender<ProgressEvent>,
    ) -> Result<()> {
        let dest_path = PathBuf::from(dest);
        let state_path = dest_path.with_extension("json");

        let state = self
            .get_or_create_state(url, &dest_path, &state_path)
            .await?;
        let state = Arc::new(RwLock::new(state));

        let (total_size, is_streaming) = {
            let s = state.read().await;
            (s.total_size, s.is_streaming)
        };

        event_tx
            .send(ProgressEvent::Initialized { total_size })
            .await?;

        if is_streaming {
            // 流式下载
            self.download_streaming(url, &dest_path, event_tx).await
        } else {
            // 分块下载
            self.download_chunked(state, &dest_path, &state_path, event_tx)
                .await
        }
    }

    /// 流式下载（不需要 Content-Length）
    async fn download_streaming(
        &self,
        url: &str,
        dest: &std::path::PathBuf,
        event_tx: mpsc::Sender<ProgressEvent>,
    ) -> Result<()> {
        let mut request = self.client.get(url);

        // 添加自定义头
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        // 添加 User-Agent
        if let Some(ua) = &self.config.user_agent {
            request = request.header(USER_AGENT, ua);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        let mut file = fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let speed_limiter = self
            .config
            .speed_limit
            .map(|limit| Arc::new(RwLock::new(SpeedLimiter::new(limit))));

        while let Some(item) = stream.next().await {
            let chunk_data = item.map_err(|e| anyhow!("Stream error: {}", e))?;
            file.write_all(&chunk_data).await?;

            let len = chunk_data.len() as u64;
            downloaded += len;

            if let Some(speed_limiter) = &speed_limiter {
                speed_limiter.write().await.wait(len).await;
            }

            let _ = event_tx
                .send(ProgressEvent::StreamDownloading { downloaded })
                .await;
        }

        file.flush().await?;
        event_tx.send(ProgressEvent::Finished).await?;
        Ok(())
    }

    /// 分块下载（需要 Content-Length）
    async fn download_chunked(
        &self,
        state: Arc<tokio::sync::RwLock<DownloadState>>,
        dest_path: &Path,
        state_path: &Path,
        event_tx: mpsc::Sender<ProgressEvent>,
    ) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));
        let speed_limiter = self
            .config
            .speed_limit
            .map(|limit| Arc::new(RwLock::new(SpeedLimiter::new(limit))));
        let mut workers = Vec::new();

        let (chunks_count, url) = {
            let s = state.read().await;
            (s.chunks.len(), s.url.clone())
        };

        for i in 0..chunks_count {
            let permit = semaphore.clone().acquire_owned().await?;
            let state_c = Arc::clone(&state);
            let client_c = self.client.clone();
            let url_c = url.clone();
            let dest_c = dest_path.to_path_buf();
            let state_file_c = state_path.to_path_buf();
            let tx_c = event_tx.clone();
            let speed_limiter_c = speed_limiter.clone();
            let headers = self.config.headers.clone();
            let user_agent = self.config.user_agent.clone();

            workers.push(tokio::spawn(async move {
                let res = Self::download_chunk(
                    i,
                    client_c,
                    &url_c,
                    &dest_c,
                    &state_file_c,
                    state_c,
                    tx_c,
                    speed_limiter_c,
                    headers,
                    user_agent,
                )
                .await;
                drop(permit);
                res
            }));
        }

        for worker in workers {
            worker.await??;
        }

        fs::remove_file(state_path).await?;
        event_tx.send(ProgressEvent::Finished).await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    /// 下载单个分块
    async fn download_chunk(
        index: usize,
        client: reqwest::Client,
        url: &str,
        dest: &Path,
        state_file: &Path,
        state_lock: Arc<tokio::sync::RwLock<DownloadState>>,
        tx: mpsc::Sender<ProgressEvent>,
        speed_limiter: Option<Arc<RwLock<SpeedLimiter>>>,
        headers: std::collections::HashMap<String, String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let (start_pos, end_pos) = {
            let s = state_lock.read().await;
            let chunk = &s.chunks[index];
            if chunk.is_finished {
                return Ok(());
            }
            (chunk.current, chunk.end)
        };

        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 5;

        loop {
            let mut request = client
                .get(url)
                .header(RANGE, format!("bytes={}-{}", start_pos, end_pos));

            // 添加自定义头
            for (key, value) in &headers {
                request = request.header(key, value);
            }

            // 添加 User-Agent
            if let Some(ua) = &user_agent {
                request = request.header(USER_AGENT, ua);
            }

            let res = request.send().await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    let mut file = fs::OpenOptions::new().write(true).open(&dest).await?;
                    file.seek(SeekFrom::Start(start_pos)).await?;

                    let mut stream = resp.bytes_stream();
                    let mut current_idx = start_pos;

                    while let Some(item) = stream.next().await {
                        let chunk_data = item.map_err(|e| anyhow!("Stream error: {}", e))?;
                        file.write_all(&chunk_data).await?;

                        let len = chunk_data.len() as u64;
                        current_idx += len;

                        if let Some(speed_limiter) = &speed_limiter {
                            speed_limiter.write().await.wait(len).await;
                        }

                        // 更新内存状态
                        {
                            let mut s = state_lock.write().await;
                            s.chunks[index].current = current_idx;
                        }

                        let _ = tx
                            .send(ProgressEvent::ChunkDownloading {
                                chunk_index: index,
                                delta: len,
                            })
                            .await;

                        // 保存状态
                        let state = state_lock.read().await;
                        state.save(state_file).await?;
                    }

                    let mut s = state_lock.write().await;
                    s.chunks[index].is_finished = true;
                    return Ok(());
                }
                _ => {
                    retry_count += 1;
                    if retry_count > MAX_RETRIES {
                        return Err(anyhow!(
                            "Chunk {} failed after {} retries",
                            index,
                            MAX_RETRIES
                        ));
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }

    /// 获取或创建下载状态
    async fn get_or_create_state(
        &self,
        url: &str,
        dest: &Path,
        state_path: &Path,
    ) -> Result<DownloadState> {
        // 尝试加载已有状态
        if let Some(state) = DownloadState::load(state_path).await?
            && state.url == url
        {
            return Ok(state);
        }

        // 检查服务器是否支持 Range 请求和 Content-Length
        let res = self.client.head(url).send().await?;
        let total_size_opt = res
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok()?.parse::<u64>().ok());

        let supports_range = res
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("").contains("bytes"))
            .unwrap_or(false);

        let use_streaming = total_size_opt.is_none() || !supports_range;

        if use_streaming {
            // 流式下载模式
            return Ok(DownloadState {
                url: url.to_string(),
                total_size: total_size_opt,
                chunks: Vec::new(),
                is_streaming: true,
            });
        }

        // 分块下载模式
        let total_size = total_size_opt.unwrap(); // 已经检查过存在

        let file = fs::File::create(dest).await?;
        file.set_len(total_size).await?;

        let mut chunks = Vec::new();
        let mut curr = 0;
        let mut idx = 0;
        while curr < total_size {
            let end = (curr + self.config.chunk_size - 1).min(total_size - 1);
            chunks.push(ChunkState {
                index: idx,
                start: curr,
                end,
                current: curr,
                is_finished: false,
            });
            curr += self.config.chunk_size;
            idx += 1;
        }

        let state = DownloadState {
            url: url.to_string(),
            total_size: Some(total_size),
            chunks,
            is_streaming: false,
        };
        state.save(state_path).await?;
        Ok(state)
    }
}
