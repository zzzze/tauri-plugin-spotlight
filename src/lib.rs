#[cfg_attr(target_os = "macos", path = "spotlight_macos/mod.rs")]
#[cfg_attr(not(target_os = "macos"), path = "spotlight_others.rs")]
mod spotlight;
mod error;
mod config;

pub use config::{PluginConfig, WindowConfig};
pub use error::Error;
pub use spotlight::SpotlightManager;

use tauri::{
    plugin::{Builder, TauriPlugin}, Manager, Runtime, State, WebviewWindow, Wry
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
fn show(manager: State<'_, spotlight::SpotlightManager>, window: WebviewWindow<Wry>) -> Result<(), String> {
    manager.show(&window).map_err(|err| format!("{:?}", err))
}

#[tauri::command]
fn hide(manager: State<'_, spotlight::SpotlightManager>, window: WebviewWindow<Wry>) -> Result<(), String> {
    manager.hide(&window).map_err(|err| format!("{:?}", err))
}

pub fn init(spotlight_config: Option<PluginConfig>) -> TauriPlugin<Wry, Option<PluginConfig>> {
    Builder::<Wry, Option<PluginConfig>>::new("spotlight")
        .invoke_handler(tauri::generate_handler![show, hide])
        .setup(|app, plugin_api| {
            app.manage(spotlight::SpotlightManager::new(
                PluginConfig::merge(
                    &spotlight_config.unwrap_or(PluginConfig::default()),
                    &plugin_api.config().clone().unwrap_or(PluginConfig::default()),
                )
            ));
            Ok(())
        })
        .build()
}
