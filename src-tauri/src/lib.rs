// Academ-IA Node — Library entry point (required by Tauri v2)
// This file exposes the run() function used by the Tauri build system.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The actual app startup is handled in main.rs
    // This lib.rs is required by Tauri v2's build system
}
