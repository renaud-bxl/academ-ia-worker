pub mod commands;
pub mod manager;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OllamaStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

impl Default for OllamaStatus {
    fn default() -> Self {
        OllamaStatus::Stopped
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaState {
    pub status: OllamaStatus,
    pub pid: Option<u32>,
    pub port: u16,
    pub version: Option<String>,
    pub error: Option<String>,
}

impl Default for OllamaState {
    fn default() -> Self {
        OllamaState {
            status: OllamaStatus::Stopped,
            pid: None,
            port: 11434,
            version: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub digest: String,
    pub modified_at: String,
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    pub model: String,
    pub status: String,
    pub completed: Option<u64>,
    pub total: Option<u64>,
    pub percent: f32,
}
