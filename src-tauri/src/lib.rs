mod config;
mod history;
mod updater;

use config::AppConfig;
use history::{CompletedTask, DownloadHistory};
use std::{path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager, State};
use tokio::sync::RwLock;
use yushi_core::{queue::DownloadQueue, types::DownloadTask};

struct AppState {
    queue: Arc<DownloadQueue>,
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    history: Arc<RwLock<DownloadHistory>>,
    history_path: PathBuf,
}

#[tauri::command]
async fn add_task(state: State<'_, AppState>, url: String, dest: String) -> Result<String, String> {
    state
        .queue
        .add_task(url, PathBuf::from(dest))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tasks(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    Ok(state.queue.get_all_tasks().await)
}

#[tauri::command]
async fn pause_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.queue.pause_task(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn resume_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .queue
        .resume_task(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cancel_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .queue
        .cancel_task(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .queue
        .remove_task(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
async fn update_config(state: State<'_, AppState>, new_config: AppConfig) -> Result<(), String> {
    // 验证配置
    new_config.validate().map_err(|e| e.to_string())?;

    // 更新内存中的配置
    let mut config = state.config.write().await;
    *config = new_config.clone();

    // 保存到文件
    new_config
        .save(&state.config_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_history(state: State<'_, AppState>) -> Result<Vec<CompletedTask>, String> {
    let history = state.history.read().await;
    Ok(history.get_all().to_vec())
}

#[tauri::command]
async fn add_to_history(state: State<'_, AppState>, task: CompletedTask) -> Result<(), String> {
    let mut history = state.history.write().await;
    history.add_completed(task);

    // 保存到文件
    history
        .save(&state.history_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn remove_from_history(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut history = state.history.write().await;

    if history.remove(&id) {
        // 保存到文件
        history
            .save(&state.history_path)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("History item not found".to_string())
    }
}

#[tauri::command]
async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let mut history = state.history.write().await;
    history.clear();

    // 保存到文件
    history
        .save(&state.history_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn search_history(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<CompletedTask>, String> {
    let history = state.history.read().await;
    Ok(history.search(&query))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_data_dir = app.path().app_data_dir().unwrap();

            // Ensure app data dir exists
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).unwrap();
            }

            let queue_path = app_data_dir.join("queue.json");
            let config_path = app_data_dir.join("config.json");
            let history_path = app_data_dir.join("history.json");

            // Load or create config
            let config = tauri::async_runtime::block_on(async {
                AppConfig::load(&config_path).await.unwrap_or_default()
            });

            // Save default config if it doesn't exist
            if !config_path.exists() {
                let _ = tauri::async_runtime::block_on(config.save(&config_path));
            }

            // Load or create history
            let history = tauri::async_runtime::block_on(async {
                DownloadHistory::load(&history_path)
                    .await
                    .unwrap_or_default()
            });

            // Initialize DownloadQueue with config
            let (queue, mut rx) = DownloadQueue::new(
                config.max_concurrent_downloads,
                config.max_concurrent_tasks,
                queue_path,
            );
            let queue = Arc::new(queue);
            let config = Arc::new(RwLock::new(config));
            let history = Arc::new(RwLock::new(history));

            // Load existing tasks
            let queue_clone = queue.clone();
            tauri::async_runtime::spawn(async move {
                let _ = queue_clone.load_from_state().await;
            });

            // Spawn event listener
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    let _ = app_handle.emit("download-event", event);
                }
            });

            app.manage(AppState {
                queue,
                config,
                config_path,
                history,
                history_path,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_task,
            get_tasks,
            pause_task,
            resume_task,
            cancel_task,
            remove_task,
            get_config,
            update_config,
            get_history,
            add_to_history,
            remove_from_history,
            clear_history,
            search_history,
            updater::check_for_updates,
            updater::download_and_install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
