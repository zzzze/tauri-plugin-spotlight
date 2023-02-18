mod spotlight;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Wry
};

pub fn init() -> TauriPlugin<Wry> {
    Builder::new("spotlight")
        .invoke_handler(tauri::generate_handler![
            spotlight::init_spotlight_window,
        ])
        .setup(|app| {
            // let w = app.get_window("main");
            app.manage(spotlight::SpotlightManager::default());
            set_state!(app, frontmost_window_path, spotlight::get_frontmost_app_path());
            Ok(())
        })
        .build()
}
