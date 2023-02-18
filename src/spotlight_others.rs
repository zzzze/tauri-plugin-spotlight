use std::sync::Mutex;
use tauri::{
    GlobalShortcutManager, Manager, Window, WindowEvent, Wry,
};
use super::Error;

#[derive(Default, Debug)]
pub struct SpotlightManager {
    close_shortcut: Option<String>,
    hide_when_inactive: bool,
    registered_window: Mutex<Vec<String>>,
}

pub struct Config {
    pub close_shortcut: Option<String>,
    pub hide_when_inactive: bool,
}

impl SpotlightManager {
    pub fn new(config: Config) -> Self {
        let mut manager = Self::default();
        manager.close_shortcut = config.close_shortcut;
        manager.hide_when_inactive = config.hide_when_inactive;
        manager
    }

    pub fn init_spotlight_window(&self, window: &Window<Wry>, shortcut: &str) -> Result<(), Error> {
        let label = window.label().to_string();
        let handle = window.app_handle();
        let state = handle.state::<SpotlightManager>();
        let mut registered_window = state
            .registered_window
            .lock()
            .map_err(|_| Error::FailedToLockMutex)?;
        let registered = registered_window.contains(&label);
        if !registered {
            register_shortcut(&window, shortcut)?;
            register_spotlight_window_backdrop(&window);
            registered_window.push(label);
        }
        Ok(())
    }
}

fn register_shortcut(window: &Window<Wry>, shortcut: &str) -> Result<(), Error> {
    let window = window.to_owned();
    let w = window.clone();
    let mut shortcut_manager = window.app_handle().global_shortcut_manager();
    shortcut_manager.register(shortcut, move || {
        if window.is_visible().unwrap() {
            window.hide().unwrap();
        } else {
            window.set_focus().unwrap();
        }
    }).map_err(|_| Error::FailedToRegisterShortcut)?;
    let app_handle = w.app_handle();
    let state = app_handle.state::<SpotlightManager>();
    if let Some(close_shortcut) = state.close_shortcut.clone() {
        if let Ok(registered) = shortcut_manager.is_registered(&close_shortcut) {
            if !registered {
                shortcut_manager.register(&close_shortcut, move || {
                    let app_handle = w.app_handle();
                    let state = app_handle.state::<SpotlightManager>();
                    let registered_window = state.registered_window.lock().unwrap();
                    let window_labels = registered_window.clone();
                    std::mem::drop(registered_window);
                    for label in window_labels {
                        if let Some(window) = app_handle.get_window(&label) {
                            if window.is_visible().unwrap() {
                                window.hide().unwrap();
                            }
                        }
                    }
                }).map_err(|_| Error::FailedToRegisterShortcut)?;
            }
        } else {
            return Err(Error::FailedToRegisterShortcut);
        }
    }
    Ok(())
}

fn register_spotlight_window_backdrop(window: &Window<Wry>) {
    let w = window.to_owned();
    let app_handle = w.app_handle();
    let state = app_handle.state::<SpotlightManager>();
    if state.hide_when_inactive {
        window.on_window_event(move |event| {
            if let WindowEvent::Focused(false) = event {
                w.hide().unwrap();
            }
        });
    }
}
