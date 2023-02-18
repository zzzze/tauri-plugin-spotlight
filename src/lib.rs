#[cfg_attr(target_os = "macos", path = "spotlight_macos.rs")]
#[cfg_attr(not(target_os = "macos"), path = "spotlight_others.rs")]
mod spotlight;
mod error;

pub use spotlight::Config;
pub use error::Error;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Wry, Runtime, State, Window
};

pub trait ManagerExt<R: Runtime> {
    fn spotlight(&self) -> State<'_, spotlight::SpotlightManager>;
}

impl<R: Runtime, T: Manager<R>> ManagerExt<R> for T {
  fn spotlight(&self) -> State<'_, spotlight::SpotlightManager> {
    self.state::<spotlight::SpotlightManager>()
  }
}

#[tauri::command]
fn show(manager: State<'_, spotlight::SpotlightManager>, window: Window<Wry>) -> Result<(), String> {
    manager.show(&window).map_err(|err| format!("{:?}", err))
}

#[tauri::command]
fn hide(manager: State<'_, spotlight::SpotlightManager>, window: Window<Wry>) -> Result<(), String> {
    manager.hide(&window).map_err(|err| format!("{:?}", err))
}

pub fn init(config: Config) -> TauriPlugin<Wry> {
    Builder::new("spotlight")
        .invoke_handler(tauri::generate_handler![show, hide])
        .setup(|app| {
            app.manage(spotlight::SpotlightManager::new(config));
            Ok(())
        })
        .build()
}
