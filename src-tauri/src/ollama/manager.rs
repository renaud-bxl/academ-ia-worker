use anyhow::{anyhow, Result};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{OllamaStatus, OllamaState};

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";

/// Check if Ollama process is running by pinging its API
pub async fn ping_ollama() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    client
        .get(format!("{}/api/version", OLLAMA_BASE_URL))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Get Ollama version string
pub async fn get_version() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let resp = client
        .get(format!("{}/api/version", OLLAMA_BASE_URL))
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;
    json["version"].as_str().map(|s| s.to_string())
}

/// Auto-start Ollama if enabled in config and not already running
pub async fn auto_start(app: &AppHandle) -> Result<()> {
    let state = app.state::<Arc<Mutex<AppState>>>();
    let config = {
        let s = state.lock().await;
        s.config.clone()
    };

    if !config.ollama_auto_start {
        tracing::info!("Ollama auto-start disabled in config");
        return Ok(());
    }

    if ping_ollama().await {
        tracing::info!("Ollama already running");
        let mut s = state.lock().await;
        s.ollama.status = OllamaStatus::Running;
        s.ollama.version = get_version().await;
        return Ok(());
    }

    tracing::info!("Starting Ollama...");
    start_ollama_process(app).await
}

/// Start the Ollama process
pub async fn start_ollama_process(app: &AppHandle) -> Result<()> {
    let state = app.state::<Arc<Mutex<AppState>>>();

    {
        let mut s = state.lock().await;
        s.ollama.status = OllamaStatus::Starting;
        s.ollama.error = None;
    }

    // Determine Ollama executable path
    let ollama_path = find_ollama_executable();

    if ollama_path.is_none() {
        let mut s = state.lock().await;
        s.ollama.status = OllamaStatus::Error;
        s.ollama.error = Some("Ollama not found. Please install Ollama from https://ollama.ai".to_string());
        return Err(anyhow!("Ollama executable not found"));
    }

    let path = ollama_path.unwrap();
    tracing::info!("Starting Ollama from: {}", path);

    // Spawn Ollama serve process
    let child = tokio::process::Command::new(&path)
        .arg("serve")
        .spawn();

    match child {
        Ok(mut process) => {
            let pid = process.id();
            {
                let mut s = state.lock().await;
                s.ollama.pid = pid;
            }

            // Wait for Ollama to be ready (max 30 seconds)
            for i in 0..30 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if ping_ollama().await {
                    let version = get_version().await;
                    let mut s = state.lock().await;
                    s.ollama.status = OllamaStatus::Running;
                    s.ollama.version = version;
                    tracing::info!("Ollama started successfully (attempt {})", i + 1);
                    return Ok(());
                }
            }

            // Timeout
            let mut s = state.lock().await;
            s.ollama.status = OllamaStatus::Error;
            s.ollama.error = Some("Ollama failed to start within 30 seconds".to_string());
            Err(anyhow!("Ollama startup timeout"))
        }
        Err(e) => {
            let mut s = state.lock().await;
            s.ollama.status = OllamaStatus::Error;
            s.ollama.error = Some(format!("Failed to start Ollama: {}", e));
            Err(anyhow!("Failed to spawn Ollama: {}", e))
        }
    }
}

/// Stop the Ollama process
pub async fn stop_ollama_process(app: &AppHandle) -> Result<()> {
    let state = app.state::<Arc<Mutex<AppState>>>();

    let pid = {
        let s = state.lock().await;
        s.ollama.pid
    };

    if let Some(pid) = pid {
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("kill").arg(pid.to_string()).spawn()?;
        }
        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .spawn()?;
        }
    }

    let mut s = state.lock().await;
    s.ollama.status = OllamaStatus::Stopped;
    s.ollama.pid = None;

    Ok(())
}

/// List installed Ollama models
pub async fn list_models() -> Result<Vec<super::ModelInfo>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let resp = client
        .get(format!("{}/api/tags", OLLAMA_BASE_URL))
        .send()
        .await?;

    let json: serde_json::Value = resp.json().await?;
    let models = json["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|m| super::ModelInfo {
            name: m["name"].as_str().unwrap_or("").to_string(),
            size: m["size"].as_u64().unwrap_or(0),
            digest: m["digest"].as_str().unwrap_or("").to_string(),
            modified_at: m["modified_at"].as_str().unwrap_or("").to_string(),
            family: m["details"]["family"].as_str().map(|s| s.to_string()),
            parameter_size: m["details"]["parameter_size"].as_str().map(|s| s.to_string()),
            quantization_level: m["details"]["quantization_level"].as_str().map(|s| s.to_string()),
        })
        .collect();

    Ok(models)
}

/// Pull (download) an Ollama model
pub async fn pull_model(model_name: &str, app: &AppHandle) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // 1 hour for large models
        .build()?;

    let resp = client
        .post(format!("{}/api/pull", OLLAMA_BASE_URL))
        .json(&serde_json::json!({ "name": model_name, "stream": false }))
        .send()
        .await?;

    if resp.status().is_success() {
        tracing::info!("Model {} pulled successfully", model_name);
        Ok(())
    } else {
        let err = resp.text().await.unwrap_or_default();
        Err(anyhow!("Failed to pull model {}: {}", model_name, err))
    }
}

/// Delete an Ollama model
pub async fn delete_model(model_name: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .delete(format!("{}/api/delete", OLLAMA_BASE_URL))
        .json(&serde_json::json!({ "name": model_name }))
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let err = resp.text().await.unwrap_or_default();
        Err(anyhow!("Failed to delete model {}: {}", model_name, err))
    }
}

/// Find Ollama executable on the system
fn find_ollama_executable() -> Option<String> {
    let candidates = if cfg!(windows) {
        vec![
            r"C:\Program Files\Ollama\ollama.exe".to_string(),
            r"C:\Users\Public\Ollama\ollama.exe".to_string(),
            "ollama.exe".to_string(),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/usr/local/bin/ollama".to_string(),
            "/opt/homebrew/bin/ollama".to_string(),
            "/Applications/Ollama.app/Contents/MacOS/ollama".to_string(),
            "ollama".to_string(),
        ]
    } else {
        vec![
            "/usr/local/bin/ollama".to_string(),
            "/usr/bin/ollama".to_string(),
            "ollama".to_string(),
        ]
    };

    for candidate in candidates {
        if std::path::Path::new(&candidate).exists() {
            return Some(candidate);
        }
        // Try which/where command
        if let Ok(output) = std::process::Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg("ollama")
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }

    None
}
