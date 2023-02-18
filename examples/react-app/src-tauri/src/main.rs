#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
use tauri_plugin_spotlight::ManagerExt;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(tauri_plugin_spotlight::Config {
            close_shortcut: Some(String::from("Escape")),
            hide_when_inactive: true,
        }))
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            if let Some(window) = app.get_window("main") {
                app.spotlight().init_spotlight_window(&window, "Ctrl+Shift+J").unwrap();
            }
            if let Some(window) = app.get_window("secondary") {
                app.spotlight().init_spotlight_window(&window, "Ctrl+Shift+K").unwrap();
            }
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
