// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ollama;
mod tunnel;
mod updater;
mod state;

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem};
use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
        )
        .init();

    // Build system tray
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Ouvrir Academ-IA Node"))
        .add_item(CustomMenuItem::new("status".to_string(), "Statut: Démarrage..."))
        .add_item(CustomMenuItem::new("separator".to_string(), "---"))
        .add_item(CustomMenuItem::new("quit".to_string(), "Quitter"));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            SystemTrayEvent::DoubleClick { .. } => {
                if let Some(window) = app.get_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            _ => {}
        })
        .manage(Arc::new(Mutex::new(AppState::default())))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            ollama::commands::get_ollama_status,
            ollama::commands::start_ollama,
            ollama::commands::stop_ollama,
            ollama::commands::list_models,
            ollama::commands::pull_model,
            ollama::commands::delete_model,
            tunnel::commands::get_tunnel_status,
            tunnel::commands::start_tunnel,
            tunnel::commands::stop_tunnel,
            updater::commands::check_for_updates,
            state::commands::get_app_state,
            state::commands::save_config,
        ])
        .setup(|app| {
            let app_handle = app.handle();

            // Start background services
            tauri::async_runtime::spawn(async move {
                // Auto-start Ollama if configured
                if let Err(e) = ollama::manager::auto_start(&app_handle).await {
                    tracing::warn!("Ollama auto-start failed: {}", e);
                }

                // Register with Academ-IA platform
                if let Err(e) = tunnel::manager::register_worker(&app_handle).await {
                    tracing::warn!("Worker registration failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Academ-IA Node");
}
