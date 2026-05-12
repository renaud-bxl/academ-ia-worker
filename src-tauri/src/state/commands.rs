use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use super::{AppState, WorkerConfig};

/// Get the full application state
#[tauri::command]
pub async fn get_app_state(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AppState, String> {
    let s = state.lock().await;
    Ok(s.clone())
}

/// Save worker configuration
#[tauri::command]
pub async fn save_config(
    config: WorkerConfig,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<WorkerConfig, String> {
    let mut s = state.lock().await;
    s.config = config.clone();
    tracing::info!("Config saved for worker: {}", config.worker_name);
    Ok(config)
}
