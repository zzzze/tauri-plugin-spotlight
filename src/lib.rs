#[cfg_attr(target_os = "macos", path = "spotlight_macos.rs")]
#[cfg_attr(not(target_os = "macos"), path = "spotlight_others.rs")]
mod spotlight;
mod error;

pub use spotlight::Config;
pub use error::Error;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Wry, Runtime, State
};

pub trait ManagerExt<R: Runtime> {
    fn spotlight(&self) -> State<'_, spotlight::SpotlightManager>;
}

impl<R: Runtime, T: Manager<R>> ManagerExt<R> for T {
  fn spotlight(&self) -> State<'_, spotlight::SpotlightManager> {
    self.state::<spotlight::SpotlightManager>()
  }
}

pub fn init(config: Config) -> TauriPlugin<Wry> {
    Builder::new("spotlight")
        .setup(|app| {
            app.manage(spotlight::SpotlightManager::new(config));
            Ok(())
        })
        .build()
}
