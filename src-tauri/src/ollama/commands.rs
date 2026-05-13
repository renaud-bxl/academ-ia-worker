use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{ModelInfo, OllamaState};

/// Get current Ollama status
#[tauri::command]
pub async fn get_ollama_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<OllamaState, String> {
    let s = state.lock().await;
    Ok(s.ollama.clone())
}

/// Start Ollama process
#[tauri::command]
pub async fn start_ollama(app: AppHandle) -> Result<OllamaState, String> {
    super::manager::start_ollama_process(&app)
        .await
        .map_err(|e| e.to_string())?;

    let state = app.state::<Arc<Mutex<AppState>>>();
    let s = state.lock().await;
    Ok(s.ollama.clone())
}

/// Stop Ollama process
#[tauri::command]
pub async fn stop_ollama(app: AppHandle) -> Result<OllamaState, String> {
    super::manager::stop_ollama_process(&app)
        .await
        .map_err(|e| e.to_string())?;

    let state = app.state::<Arc<Mutex<AppState>>>();
    let s = state.lock().await;
    Ok(s.ollama.clone())
}

/// List installed models
#[tauri::command]
pub async fn list_models() -> Result<Vec<ModelInfo>, String> {
    super::manager::list_models()
        .await
        .map_err(|e| e.to_string())
}

/// Pull (download) a model
#[tauri::command]
pub async fn pull_model(model: String, app: AppHandle) -> Result<String, String> {
    super::manager::pull_model(&model, &app)
        .await
        .map(|_| format!("Model {} downloaded successfully", model))
        .map_err(|e| e.to_string())
}

/// Delete a model
#[tauri::command]
pub async fn delete_model(model: String) -> Result<String, String> {
    super::manager::delete_model(&model)
        .await
        .map(|_| format!("Model {} deleted", model))
        .map_err(|e| e.to_string())
}
