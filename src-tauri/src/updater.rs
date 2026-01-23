use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

/// 更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 是否有可用更新
    pub available: bool,
    /// 当前版本
    pub current_version: String,
    /// 最新版本
    pub latest_version: Option<String>,
    /// 更新日志
    pub body: Option<String>,
    /// 发布日期
    pub date: Option<String>,
}

/// 检查更新
#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();

    // 使用 tauri-plugin-updater 检查更新
    let updater = app
        .updater()
        .map_err(|e| format!("初始化更新器失败: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let body = update.body.clone();
            let date = update.date.map(|d| d.to_string());

            Ok(UpdateInfo {
                available: true,
                current_version,
                latest_version: Some(version),
                body,
                date,
            })
        }
        Ok(None) => Ok(UpdateInfo {
            available: false,
            current_version,
            latest_version: None,
            body: None,
            date: None,
        }),
        Err(e) => Err(format!("检查更新失败: {}", e)),
    }
}

/// 下载并安装更新
#[tauri::command]
pub async fn download_and_install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app
        .updater()
        .map_err(|e| format!("初始化更新器失败: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            // 发送下载开始事件
            let _ = app.emit("update-download-started", ());

            // 下载更新
            let mut downloaded = 0usize;

            match update
                .download(
                    |chunk_length, content_length| {
                        downloaded += chunk_length;
                        // 发送下载进度事件
                        let progress = if let Some(total) = content_length {
                            (downloaded as f64 / total as f64 * 100.0) as u32
                        } else {
                            0
                        };
                        let _ = app.emit("update-download-progress", progress);
                    },
                    || {
                        // 下载完成
                        let _ = app.emit("update-download-finished", ());
                    },
                )
                .await
            {
                Ok(bytes) => {
                    // 安装更新（会重启应用）
                    let _ = app.emit("update-installing", ());
                    update
                        .install(bytes)
                        .map_err(|e| format!("安装更新失败: {}", e))?;
                    Ok(())
                }
                Err(e) => Err(format!("下载更新失败: {}", e)),
            }
        }
        Ok(None) => Err("没有可用的更新".to_string()),
        Err(e) => Err(format!("检查更新失败: {}", e)),
    }
}
