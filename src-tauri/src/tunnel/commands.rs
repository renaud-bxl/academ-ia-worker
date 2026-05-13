use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

use crate::state::AppState;
use super::TunnelState;

/// Get current tunnel status
#[tauri::command]
pub async fn get_tunnel_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<TunnelState, String> {
    let s = state.lock().await;
    Ok(s.tunnel.clone())
}

/// Connect to the Academ-IA platform
#[tauri::command]
pub async fn start_tunnel(app: AppHandle) -> Result<TunnelState, String> {
    super::manager::register_worker(&app)
        .await
        .map_err(|e| e.to_string())?;

    let state = app.state::<Arc<Mutex<AppState>>>();
    let s = state.lock().await;
    Ok(s.tunnel.clone())
}

/// Disconnect from the platform
#[tauri::command]
pub async fn stop_tunnel(app: AppHandle) -> Result<TunnelState, String> {
    super::manager::disconnect(&app)
        .await
        .map_err(|e| e.to_string())?;

    let state = app.state::<Arc<Mutex<AppState>>>();
    let s = state.lock().await;
    Ok(s.tunnel.clone())
}
