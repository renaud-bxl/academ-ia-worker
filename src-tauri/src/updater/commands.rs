use super::UpdateInfo;

/// Check for available updates via GitHub Releases
#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("academ-ia-worker")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/renaud-bxl/academ-ia-worker/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Failed to check updates: {}", e))?;

    if !resp.status().is_success() {
        return Ok(UpdateInfo {
            available: false,
            current_version,
            latest_version: None,
            release_notes: None,
            download_url: None,
        });
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let latest_version = json["tag_name"]
        .as_str()
        .map(|v| v.trim_start_matches('v').to_string());

    let release_notes = json["body"].as_str().map(|s| s.to_string());

    // Find the appropriate asset for this platform
    let platform_suffix = if cfg!(windows) {
        "_x64-setup.exe"
    } else if cfg!(target_os = "macos") {
        ".dmg"
    } else {
        ".AppImage"
    };

    let download_url = json["assets"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n.ends_with(platform_suffix))
                .unwrap_or(false)
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .map(|s| s.to_string());

    let available = latest_version
        .as_ref()
        .map(|v| v != &current_version)
        .unwrap_or(false);

    Ok(UpdateInfo {
        available,
        current_version,
        latest_version,
        release_notes,
        download_url,
    })
}
