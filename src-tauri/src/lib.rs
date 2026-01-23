use std::{path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager, State};
use yushi_core::{queue::DownloadQueue, types::DownloadTask};

struct AppState {
    queue: Arc<DownloadQueue>,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_data_dir = app.path().app_data_dir().unwrap();

            // Ensure app data dir exists
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).unwrap();
            }

            let queue_path = app_data_dir.join("queue.json");

            // Initialize DownloadQueue
            // max_concurrent_downloads: 4, max_concurrent_tasks: 3
            let (queue, mut rx) = DownloadQueue::new(4, 3, queue_path);
            let queue = Arc::new(queue);

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

            app.manage(AppState { queue });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_task,
            get_tasks,
            pause_task,
            resume_task,
            cancel_task,
            remove_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
