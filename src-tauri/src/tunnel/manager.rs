use anyhow::{anyhow, Result};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use crate::state::AppState;
use super::TunnelStatus;

/// Register this worker with the Academ-IA platform
pub async fn register_worker(app: &AppHandle) -> Result<()> {
    let state = app.state::<Arc<Mutex<AppState>>>();
    let config = {
        let s = state.lock().await;
        s.config.clone()
    };

    // Check if we have registration credentials
    if config.api_key.is_empty() || config.organization_id.is_empty() {
        tracing::info!("Worker not configured yet — skipping registration");
        return Ok(());
    }

    {
        let mut s = state.lock().await;
        s.tunnel.status = TunnelStatus::Connecting;
    }

    let platform_url = {
        let s = state.lock().await;
        s.tunnel.platform_url.clone()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // Get local machine info
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());

    let worker_name = if config.worker_name.is_empty() {
        hostname.clone()
    } else {
        config.worker_name.clone()
    };

    // Register with platform
    let payload = serde_json::json!({
        "workerId": config.worker_id,
        "workerName": worker_name,
        "organizationId": config.organization_id,
        "ollamaUrl": format!("http://127.0.0.1:{}", config.ollama_port),
        "hostname": hostname,
        "platform": std::env::consts::OS,
        "version": env!("CARGO_PKG_VERSION"),
    });

    let resp = client
        .post(format!("{}/api/trpc/workers.register", platform_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let worker_id = json["result"]["data"]["workerId"]
                .as_str()
                .unwrap_or(&config.worker_id)
                .to_string();

            let mut s = state.lock().await;
            s.tunnel.status = TunnelStatus::Connected;
            s.tunnel.worker_id = Some(worker_id);
            s.tunnel.worker_name = Some(worker_name);
            s.tunnel.organization_id = Some(config.organization_id.clone());
            s.tunnel.last_heartbeat = Some(chrono::Utc::now().to_rfc3339());

            tracing::info!("Worker registered successfully with platform");

            // Start heartbeat loop
            let app_clone = app.clone();
            tokio::spawn(async move {
                heartbeat_loop(app_clone).await;
            });

            Ok(())
        }
        Ok(r) => {
            let status = r.status();
            let body = r.text().await.unwrap_or_default();
            let err = format!("Registration failed: HTTP {} — {}", status, body);
            tracing::warn!("{}", err);

            let mut s = state.lock().await;
            s.tunnel.status = TunnelStatus::Error;
            s.tunnel.error = Some(err.clone());

            Err(anyhow!(err))
        }
        Err(e) => {
            let err = format!("Cannot reach platform: {}", e);
            tracing::warn!("{}", err);

            let mut s = state.lock().await;
            s.tunnel.status = TunnelStatus::Error;
            s.tunnel.error = Some(err.clone());

            Err(anyhow!(err))
        }
    }
}

/// Send periodic heartbeats to keep the worker alive on the platform
async fn heartbeat_loop(app: AppHandle) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        interval.tick().await;

        let state = app.state::<Arc<Mutex<AppState>>>();
        let (config, tunnel_status) = {
            let s = state.lock().await;
            (s.config.clone(), s.tunnel.status.clone())
        };

        if tunnel_status != TunnelStatus::Connected {
            break;
        }

        if config.api_key.is_empty() {
            break;
        }

        let platform_url = {
            let s = state.lock().await;
            s.tunnel.platform_url.clone()
        };

        // Get Ollama status for heartbeat
        let ollama_running = crate::ollama::manager::ping_ollama().await;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let payload = serde_json::json!({
            "workerId": config.worker_id,
            "ollamaStatus": if ollama_running { "running" } else { "stopped" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        match client
            .post(format!("{}/api/trpc/workers.heartbeat", platform_url))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&payload)
            .send()
            .await
        {
            Ok(_) => {
                let mut s = state.lock().await;
                s.tunnel.last_heartbeat = Some(chrono::Utc::now().to_rfc3339());
            }
            Err(e) => {
                tracing::warn!("Heartbeat failed: {}", e);
            }
        }
    }
}

/// Disconnect from the platform
pub async fn disconnect(app: &AppHandle) -> Result<()> {
    let state = app.state::<Arc<Mutex<AppState>>>();
    let mut s = state.lock().await;
    s.tunnel.status = TunnelStatus::Disconnected;
    s.tunnel.worker_id = None;
    s.tunnel.last_heartbeat = None;
    Ok(())
}
