pub mod nbyte;

use anyhow::{Result, anyhow};
use fs_err::tokio as fs;
use futures::StreamExt;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncSeekExt, AsyncWriteExt, SeekFrom},
    sync::{Semaphore, mpsc},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChunkState {
    index: usize,
    start: u64,
    end: u64,
    current: u64,
    is_finished: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadState {
    url: String,
    total_size: u64,
    chunks: Vec<ChunkState>,
}

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    Initialized { total_size: u64 },
    ChunkUpdated { chunk_index: usize, delta: u64 },
    Finished,
    Failed(String),
}

pub struct YuShi {
    client: reqwest::Client,
    max_concurrent: usize,
    chunk_size: u64,
}

impl YuShi {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            client: reqwest::Client::builder()
                .tcp_keepalive(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
            max_concurrent,
            chunk_size: 10 * 1024 * 1024,
        }
    }

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
        let state = Arc::new(tokio::sync::RwLock::new(state));

        let total_size = state.read().await.total_size;
        let _ = event_tx
            .send(ProgressEvent::Initialized { total_size })
            .await;

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut workers = Vec::new();

        let chunks_count = { state.read().await.chunks.len() };
        for i in 0..chunks_count {
            let permit = semaphore.clone().acquire_owned().await?;
            let state_c = Arc::clone(&state);
            let client_c = self.client.clone();
            let url_c = url.to_string();
            let dest_c = dest_path.clone();
            let state_file_c = state_path.clone();
            let tx_c = event_tx.clone();

            workers.push(tokio::spawn(async move {
                let res =
                    Self::download_chunk(i, client_c, url_c, dest_c, state_file_c, state_c, tx_c)
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

    async fn download_chunk(
        index: usize,
        client: reqwest::Client,
        url: String,
        dest: PathBuf,
        state_file: PathBuf,
        state_lock: Arc<tokio::sync::RwLock<DownloadState>>,
        tx: mpsc::Sender<ProgressEvent>,
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
            let res = client
                .get(&url)
                .header(RANGE, format!("bytes={}-{}", start_pos, end_pos))
                .send()
                .await;

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

                        // 更新内存状态
                        {
                            let mut s = state_lock.write().await;
                            s.chunks[index].current = current_idx;
                        }

                        let _ = tx
                            .send(ProgressEvent::ChunkUpdated {
                                chunk_index: index,
                                delta: len,
                            })
                            .await;

                        Self::save_state(&state_file, &*state_lock.read().await).await?;
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

    async fn get_or_create_state(
        &self,
        url: &str,
        dest: &Path,
        state_path: &Path,
    ) -> Result<DownloadState> {
        if state_path.exists() {
            let content = fs::read_to_string(state_path).await?;
            if let Ok(state) = serde_json::from_str::<DownloadState>(&content)
                && state.url == url
            {
                return Ok(state);
            }
        }

        let res = self.client.head(url).send().await?;
        let total_size = res
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok()?.parse::<u64>().ok())
            .ok_or_else(|| anyhow!("Server must support Content-Length"))?;

        let file = fs::File::create(dest).await?;
        file.set_len(total_size).await?;

        let mut chunks = Vec::new();
        let mut curr = 0;
        let mut idx = 0;
        while curr < total_size {
            let end = (curr + self.chunk_size - 1).min(total_size - 1);
            chunks.push(ChunkState {
                index: idx,
                start: curr,
                end,
                current: curr,
                is_finished: false,
            });
            curr += self.chunk_size;
            idx += 1;
        }

        let state = DownloadState {
            url: url.to_string(),
            total_size,
            chunks,
        };
        Self::save_state(state_path, &state).await?;
        Ok(state)
    }

    async fn save_state(path: &Path, state: &DownloadState) -> Result<()> {
        let data = serde_json::to_string(state)?;
        fs::write(path, data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download() -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1024);
        let downloader = YuShi::new(4);

        // 观察者线程
        tokio::spawn(async move {
            let mut total = 0;
            let mut current = 0;
            while let Some(event) = rx.recv().await {
                match event {
                    ProgressEvent::Initialized { total_size } => total = total_size,
                    ProgressEvent::ChunkUpdated { delta, .. } => {
                        current += delta;
                        println!("Progress: {:.2}%", (current as f64 / total as f64) * 100.0);
                    }
                    ProgressEvent::Finished => println!("Done!"),
                    ProgressEvent::Failed(e) => eprintln!("Error: {}", e),
                }
            }
        });

        downloader
            .download("https://speed.hetzner.de", "video.mp4", tx)
            .await?;

        Ok(())
    }
}
