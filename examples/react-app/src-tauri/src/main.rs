#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(Some(tauri_plugin_spotlight::PluginConfig {
            windows: Some(vec![
                tauri_plugin_spotlight::WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+Shift+J"),
                },
            ]),
            close_shortcut: Some(String::from("Escape")),
            hide_when_inactive: Some(true),
        })))
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
