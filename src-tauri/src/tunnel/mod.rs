pub mod commands;
pub mod manager;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl Default for TunnelStatus {
    fn default() -> Self {
        TunnelStatus::Disconnected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelState {
    pub status: TunnelStatus,
    pub worker_id: Option<String>,
    pub worker_name: Option<String>,
    pub organization_id: Option<String>,
    pub platform_url: String,
    pub last_heartbeat: Option<String>,
    pub error: Option<String>,
}

impl Default for TunnelState {
    fn default() -> Self {
        TunnelState {
            status: TunnelStatus::Disconnected,
            worker_id: None,
            worker_name: None,
            organization_id: None,
            platform_url: "https://acad-ia-78kiixzl.manus.space".to_string(),
            last_heartbeat: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegistration {
    pub worker_id: String,
    pub worker_name: String,
    pub organization_id: String,
    pub api_key: String,
    pub ollama_url: String,
}
