#[cfg_attr(target_os = "macos", path = "spotlight_macos.rs")]
#[cfg_attr(not(target_os = "macos"), path = "spotlight_others.rs")]
mod spotlight;
mod error;
mod config;

pub use config::{PluginConfig, WindowConfig};
pub use error::Error;

use spotlight::SpotlightManager;
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

pub fn init(spotlight_config: Option<PluginConfig>) -> TauriPlugin<Wry, Option<PluginConfig>> {
    Builder::<Wry, Option<PluginConfig>>::new("spotlight")
        .invoke_handler(tauri::generate_handler![show, hide])
        .setup_with_config(|app, config| {
            app.manage(spotlight::SpotlightManager::new(
                PluginConfig::merge(
                    &spotlight_config.unwrap_or(PluginConfig::default()),
                    &config.unwrap_or(PluginConfig::default()),
                )
            ));
            Ok(())
        })
        .on_webview_ready(move |window| {
            let app_handler = window.app_handle();
            let manager = app_handler.state::<SpotlightManager>();
            if let Some(window_configs) = &manager.config.windows {
                for window_config in window_configs {
                    if window.label() == window_config.label {
                        app_handler.spotlight().init_spotlight_window(&window, &window_config.shortcut).unwrap();
                    }
                }
            }
        })
        .build()
}
