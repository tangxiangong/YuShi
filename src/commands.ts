import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  CompletedTask,
  DownloadTask,
  UpdateInfo,
} from "./types.ts";

/**
 * Tauri Commands
 * Type-safe wrappers for all Tauri backend commands
 */

/**
 * Add a new download task
 * @param url - The URL to download from
 * @param dest - The destination path to save the file
 * @returns The task ID
 */
export function addTask(url: string, dest: string): Promise<string> {
  return invoke<string>("add_task", { url, dest });
}

/**
 * Get all download tasks
 * @returns Array of all download tasks
 */
export function getTasks(): Promise<DownloadTask[]> {
  return invoke<DownloadTask[]>("get_tasks");
}

/**
 * Pause a download task
 * @param id - The task ID to pause
 */
export function pauseTask(id: string): Promise<void> {
  return invoke<void>("pause_task", { id });
}

/**
 * Resume a paused download task
 * @param id - The task ID to resume
 */
export function resumeTask(id: string): Promise<void> {
  return invoke<void>("resume_task", { id });
}

/**
 * Cancel a download task
 * @param id - The task ID to cancel
 */
export function cancelTask(id: string): Promise<void> {
  return invoke<void>("cancel_task", { id });
}

/**
 * Remove a download task from the queue
 * @param id - The task ID to remove
 */
export function removeTask(id: string): Promise<void> {
  return invoke<void>("remove_task", { id });
}

/**
 * Get application configuration
 * @returns Current application configuration
 */
export function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

/**
 * Update application configuration
 * @param config - New configuration to apply
 */
export function updateConfig(config: AppConfig): Promise<void> {
  return invoke<void>("update_config", { newConfig: config });
}

/**
 * Get download history
 * @returns Array of completed tasks
 */
export function getHistory(): Promise<CompletedTask[]> {
  return invoke<CompletedTask[]>("get_history");
}

/**
 * Add a completed task to history
 * @param task - Completed task to add
 */
export function addToHistory(task: CompletedTask): Promise<void> {
  return invoke<void>("add_to_history", { task });
}

/**
 * Remove a task from history
 * @param id - History item ID to remove
 */
export function removeFromHistory(id: string): Promise<void> {
  return invoke<void>("remove_from_history", { id });
}

/**
 * Clear all download history
 */
export function clearHistory(): Promise<void> {
  return invoke<void>("clear_history");
}

/**
 * Search download history
 * @param query - Search query string
 * @returns Matching history items
 */
export function searchHistory(query: string): Promise<CompletedTask[]> {
  return invoke<CompletedTask[]>("search_history", { query });
}

/**
 * Check for application updates
 * @returns Update information
 */
export function checkForUpdates(): Promise<UpdateInfo> {
  return invoke<UpdateInfo>("check_for_updates");
}

/**
 * Download and install update
 */
export function downloadAndInstallUpdate(): Promise<void> {
  return invoke<void>("download_and_install_update");
}
