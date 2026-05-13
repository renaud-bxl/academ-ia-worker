// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ollama;
mod tunnel;
mod updater;
mod state;

use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
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

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(Arc::new(Mutex::new(AppState::default())))
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
            // Build system tray menu
            let show_item = MenuItem::with_id(app, "show", "Ouvrir Academ-IA Node", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let app_handle = app.handle().clone();

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
