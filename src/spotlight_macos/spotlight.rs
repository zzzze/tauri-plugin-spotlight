use std::{collections::HashMap, str::FromStr, sync::{Mutex, RwLock}};
use cocoa::{
    appkit::{CGFloat, NSWindow},
    base::{id, nil, BOOL, NO, YES},
    foundation::{NSPoint, NSRect},
};
use objc_id::ShareId;
use objc::{class, msg_send, sel, sel_impl};
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent, Wry};
use super::panel::{create_spotlight_panel, RawNSPanel};
use crate::{PluginConfig, WindowConfig};
use crate::Error;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tracing::error;

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    pub fn NSMouseInRect(aPoint: NSPoint, aRect: NSRect, flipped: BOOL) -> BOOL;
}

#[derive(Default, Debug)]
pub struct SpotlightManager {
    pub config: PluginConfig,
    registered_panels: RwLock<HashMap<String, Mutex<ShareId<RawNSPanel>>>>,
}

impl SpotlightManager {
    pub fn new(config: PluginConfig) -> Self {
        let mut manager = Self::default();
        manager.config = config;
        manager
    }

    fn get_window_config(&self, window: &WebviewWindow<Wry>) -> Option<WindowConfig> {
        if let Some(window_configs) = self.config.windows.clone() {
            for window_config in window_configs {
                if window.label() == window_config.label {
                    return Some(window_config.clone());
                }
            }
        }
        None
    }

    pub fn init_spotlight_window(&self, window: &WebviewWindow<Wry>, window_config: WindowConfig) -> Result<(), Error> {
        // let window_config = match self.get_window_config(&window) {
        //     Some(window_config) => window_config,
        //     None => return Ok(()),
        // };
        let label = window.label();
        let mut map = self.registered_panels.write().map_err(|_| Error::RwLock(String::from("failed to write registered panels")))?;
        if map.get(label).is_none() {
            map.insert(String::from(label), Mutex::new(create_spotlight_panel(window)));
            if !window_config.shortcut.is_empty() {
                register_shortcut_for_window(&window, &window_config)?;
                register_close_shortcut(&window)?;
            }
            handle_focus_state_change(&window);
            set_window_level(&window, &window_config)?;
        }
        Ok(())
    }

    pub fn show(&self, window: &WebviewWindow<Wry>) -> Result<(), Error> {
        position_window_at_the_center_of_the_monitor_with_cursor(&window)?;
        let label = window.label();
        let map = self.registered_panels.read().map_err(|_| Error::RwLock(String::from("failed to read registered panels")))?;
        if let Some(panel) = map.get(label) {
            let panel = panel.lock().map_err(|_| Error::Mutex(String::from("failed to lock panel")))?;
            panel.show();
        }
        Ok(())
    }

    pub fn hide(&self, window: &WebviewWindow<Wry>) -> Result<(), Error> {
        let label = window.label();
        let map = self.registered_panels.read().map_err(|_| Error::RwLock(String::from("failed to read registered panels")))?;
        if let Some(panel) = map.get(label) {
            let panel = panel.lock().map_err(|_| Error::Mutex(String::from("failed to lock panel")))?;
            panel.order_out(None);
        }
        Ok(())
    }
}

fn set_window_level(window: &WebviewWindow<Wry>, window_config: &WindowConfig) -> Result<(), Error> {
    if let Some(level) = window_config.macos_window_level {
        let handle: id = window.ns_window().map_err(|_| Error::FailedToGetNSWindow)? as _;
        unsafe { handle.setLevel_((level).into()) };
    }
    Ok(())
}

#[macro_export]
macro_rules! nsstring_to_string {
    ($ns_string:expr) => {{
        use objc::{sel, sel_impl};
        let utf8: id = objc::msg_send![$ns_string, UTF8String];
        let string = if !utf8.is_null() {
            Some({
                std::ffi::CStr::from_ptr(utf8 as *const std::ffi::c_char)
                    .to_string_lossy()
                    .into_owned()
            })
        } else {
            None
        };
        string
    }};
}

fn register_shortcut_for_window(window: &WebviewWindow<Wry>, window_config: &WindowConfig) -> Result<(), Error> {
    let window_label = window.label().to_string();
    let shortcut_manager = window.app_handle().global_shortcut();
    let shortcut = Shortcut::from_str(&window_config.shortcut).map_err(tauri_plugin_global_shortcut::Error::from)?;
    shortcut_manager.on_shortcut(shortcut, move |app_handle, _, event| {
        if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
            return;
        }
        let manager = app_handle.state::<SpotlightManager>();
        if let Some(window) = app_handle.get_webview_window(&window_label) {
            match window.is_visible() {
                Ok(is_visible) => {
                    if is_visible {
                        manager.hide(&window).unwrap();
                    } else {
                        manager.show(&window).unwrap();
                    }
                },
                Err(err) => {
                    error!("failed to get window visibility: {}", err);
                },
            }
        }
    }).map_err(|_| Error::Other(String::from("failed to register shortcut")))?;
    Ok(())
}

fn register_close_shortcut(window: &WebviewWindow<Wry>) -> Result<(), Error> {
    let window = window.to_owned();
    let shortcut_manager = window.app_handle().global_shortcut();
    let app_handle = window.app_handle();
    let manager = app_handle.state::<SpotlightManager>();
    if let Some(close_shortcut) = &manager.config.global_close_shortcut {
        let shortcut = Shortcut::from_str(&close_shortcut).map_err(tauri_plugin_global_shortcut::Error::from)?;
        if !shortcut_manager.is_registered(shortcut.clone()) {
            shortcut_manager.on_shortcut(shortcut.clone(), |app_handle, _, _| {
                let state = app_handle.state::<SpotlightManager>();
                let labels = if let Some(ref windows) = state.config.windows {
                    windows.iter().map(|window| window.label.clone()).collect()
                } else {
                    vec![]
                };
                for label in labels {
                    if let Some(window) = app_handle.get_webview_window(&label) {
                        state.hide(&window).unwrap();
                    }
                }
            })?;
        }
    }
    Ok(())
}

fn unregister_close_shortcut(window: &WebviewWindow<Wry>) -> Result<(), Error> {
    let window = window.to_owned();
    let shortcut_manager = window.app_handle().global_shortcut();
    let app_handle = window.app_handle();
    let manager = app_handle.state::<SpotlightManager>();
    if let Some(close_shortcut) = manager.config.global_close_shortcut.clone() {
        let shortcut = Shortcut::from_str(&close_shortcut).map_err(tauri_plugin_global_shortcut::Error::from)?;
        if shortcut_manager.is_registered(shortcut.clone()) {
            shortcut_manager.unregister(shortcut.clone())?;
        }
    }
    Ok(())
}

fn handle_focus_state_change(window: &WebviewWindow<Wry>) {
    let w = window.to_owned();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            unregister_close_shortcut(&w).unwrap(); // FIXME:
            w.hide().unwrap();
        } else {
            register_close_shortcut(&w).unwrap(); // FIXME:
        }
    });
}

/// Positions a given window at the center of the monitor with cursor
fn position_window_at_the_center_of_the_monitor_with_cursor(window: &WebviewWindow<Wry>) -> Result<(), Error> {
    if let Some(monitor) = get_monitor_with_cursor() {
        let display_size = monitor.size.to_logical::<f64>(monitor.scale_factor);
        let display_pos = monitor.position.to_logical::<f64>(monitor.scale_factor);
        let handle: id = window.ns_window().map_err(|_| Error::FailedToGetNSWindow)? as _;
        let win_frame: NSRect = unsafe { handle.frame() };
        let rect = NSRect {
            origin: NSPoint {
                x: (display_pos.x + (display_size.width / 2.0)) - (win_frame.size.width / 2.0),
                y: (display_pos.y + (display_size.height / 2.0)) - (win_frame.size.height / 2.0),
            },
            size: win_frame.size,
        };
        let _: () = unsafe { msg_send![handle, setFrame: rect display: YES] };
    }
    Ok(())
}

struct Monitor {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub size: PhysicalSize<u32>,
    pub position: PhysicalPosition<i32>,
    pub scale_factor: f64,
}

/// Returns the Monitor with cursor
fn get_monitor_with_cursor() -> Option<Monitor> {
    objc::rc::autoreleasepool(|| {
        let mouse_location: NSPoint = unsafe { msg_send![class!(NSEvent), mouseLocation] };
        let screens: id = unsafe { msg_send![class!(NSScreen), screens] };
        let screens_iter: id = unsafe { msg_send![screens, objectEnumerator] };
        let mut next_screen: id;

        let frame_with_cursor: Option<NSRect> = loop {
            next_screen = unsafe { msg_send![screens_iter, nextObject] };
            if next_screen == nil {
                break None;
            }

            let frame: NSRect = unsafe { msg_send![next_screen, frame] };
            let is_mouse_in_screen_frame: BOOL =
                unsafe { NSMouseInRect(mouse_location, frame, NO) };
            if is_mouse_in_screen_frame == YES {
                break Some(frame);
            }
        };

        if let Some(frame) = frame_with_cursor {
            let name: id = unsafe { msg_send![next_screen, localizedName] };
            let screen_name = unsafe { nsstring_to_string!(name) };
            let scale_factor: CGFloat = unsafe { msg_send![next_screen, backingScaleFactor] };
            let scale_factor: f64 = scale_factor;

            return Some(Monitor {
                name: screen_name,
                position: PhysicalPosition {
                    x: (frame.origin.x * scale_factor) as i32,
                    y: (frame.origin.y * scale_factor) as i32,
                },
                size: PhysicalSize {
                    width: (frame.size.width * scale_factor) as u32,
                    height: (frame.size.height * scale_factor) as u32,
                },
                scale_factor,
            });
        }

        None
    })
}
