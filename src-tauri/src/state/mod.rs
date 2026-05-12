pub mod commands;

use serde::{Deserialize, Serialize};
use crate::ollama::OllamaState;
use crate::tunnel::TunnelState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Worker unique ID (generated on first run)
    pub worker_id: String,
    /// Human-readable name for this worker
    pub worker_name: String,
    /// Organization ID from Academ-IA platform
    pub organization_id: String,
    /// API key for platform authentication
    pub api_key: String,
    /// Ollama port (default: 11434)
    pub ollama_port: u16,
    /// Auto-start Ollama on app launch
    pub ollama_auto_start: bool,
    /// Auto-start app on system boot
    pub app_autostart: bool,
    /// Minimize to tray on close
    pub minimize_to_tray: bool,
    /// Platform URL override (for self-hosted)
    pub platform_url_override: String,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        WorkerConfig {
            worker_id: uuid::Uuid::new_v4().to_string(),
            worker_name: String::new(),
            organization_id: String::new(),
            api_key: String::new(),
            ollama_port: 11434,
            ollama_auto_start: true,
            app_autostart: false,
            minimize_to_tray: true,
            platform_url_override: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub ollama: OllamaState,
    pub tunnel: TunnelState,
    pub config: WorkerConfig,
    pub app_version: String,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            ollama: OllamaState::default(),
            tunnel: TunnelState::default(),
            config: WorkerConfig::default(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
