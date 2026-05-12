pub mod commands;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
}
