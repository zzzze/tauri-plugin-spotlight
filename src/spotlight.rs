use std::sync::Mutex;
use cocoa::{
    appkit::{CGFloat, NSMainMenuWindowLevel, NSWindow, NSWindowCollectionBehavior},
    base::{id, nil, BOOL, NO, YES},
    foundation::{NSPoint, NSRect},
};
use objc::{class, msg_send, sel, sel_impl};
use tauri::{
    GlobalShortcutManager, Manager, PhysicalPosition, PhysicalSize, Window, WindowEvent, Wry,
};
use objc::runtime::{Class, Object, Sel};

#[derive(Default)]
pub struct Store {
    pub frontmost_window_path: Option<String>,
}

#[derive(Default)]
pub struct SpotlightManager {
    pub store: Mutex<Store>,
}

#[macro_export]
macro_rules! set_state {
    ($app_handle:expr, $field:ident, $value:expr) => {{
        let handle = $app_handle.app_handle();
        handle
            .state::<$crate::spotlight::SpotlightManager>()
            .store
            .lock()
            .unwrap()
            .$field = $value;
    }};
}

#[macro_export]
macro_rules! get_state {
    ($app_handle:expr, $field:ident) => {{
        let handle = $app_handle.app_handle();
        let value = handle
            .state::<$crate::spotlight::SpotlightManager>()
            .store
            .lock()
            .unwrap()
            .$field;
        value
    }};
    ($app_handle:expr, $field:ident, $action:ident) => {{
        let handle = $app_handle.app_handle();
        let value = handle
            .state::<$crate::spotlight::SpotlightManager>()
            .store
            .lock()
            .unwrap()
            .$field
            .$action();
        value
    }};
}

#[macro_export]
macro_rules! nsstring_to_string {
    ($ns_string:expr) => {{
        use objc::{sel, sel_impl};
        let utf8: id = unsafe { objc::msg_send![$ns_string, UTF8String] };
        let string = if !utf8.is_null() {
            Some(unsafe {
                {
                    std::ffi::CStr::from_ptr(utf8 as *const std::ffi::c_char)
                        .to_string_lossy()
                        .into_owned()
                }
            })
        } else {
            None
        };

        string
    }};
}

fn switch_to_app(bundle_url: &str) {
    let workspace = unsafe {
        let workspace_class = Class::get("NSWorkspace").unwrap();
        let shared_workspace_selector = Sel::register("sharedWorkspace");
        let shared_workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        shared_workspace
    };

    let running_apps = unsafe {
        let running_apps_selector = Sel::register("runningApplications");
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        running_apps
    };

    let target_app_bundle_url = bundle_url;

    let target_app = unsafe {
        let count = msg_send![running_apps, count];
        (|| {
            for i in 0..count {
                let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
                let app_bundle_url: id = msg_send![app, bundleURL];
                let path: id = msg_send![app_bundle_url, path];
                let app_bundle_url_str = nsstring_to_string!(path);

                if let Some(app_bundle_url_str) = app_bundle_url_str {
                    if app_bundle_url_str == target_app_bundle_url.to_string() {
                        return app;
                    }
                }
            }
            let ns_object_class = Class::get("NSObject").unwrap();
            let alloc_selector = Sel::register("alloc");
            msg_send![ns_object_class, alloc]
        })()
    };

    let _: () = unsafe {
        let activate_selector = Sel::register("activateWithOptions:");
        let _: () = msg_send![target_app, activateWithOptions: 1];
        return ();
    };
}




#[tauri::command]
pub fn init_spotlight_window(window: Window<Wry>) {
    register_shortcut(&window);
    register_spotlight_window_backdrop(&window);
    set_spotlight_window_collection_behavior(&window);
    set_window_above_menubar(&window);
}

fn register_shortcut(window: &Window<Wry>) {
    let window = window.to_owned();
    let mut shortcut_manager = window.app_handle().global_shortcut_manager();

    let handle = window.app_handle();
    if let Err(e) = shortcut_manager.register("Ctrl+Shift+J", move || {
        position_window_at_the_center_of_the_monitor_with_cursor(&window);
        if window.is_visible().unwrap() {
            window.hide().unwrap();
            let w = window.to_owned();
            if let Some(prev_frontmost_window_path) = get_state!(w.app_handle(), frontmost_window_path, clone) {
                println!("prev_frontmost_window_path: {:?}", prev_frontmost_window_path);
                switch_to_app(&prev_frontmost_window_path);
            }


        } else {
            set_state!(handle, frontmost_window_path, get_frontmost_app_path());
            println!("frontmost window path {:?}", get_frontmost_app_path());
            window.set_focus().unwrap();
        }
    }) {
        println!("err: {}", e);
    }
}

fn register_spotlight_window_backdrop(window: &Window<Wry>) {
    let w = window.to_owned();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            w.hide().unwrap();
        }
    });
}

/// Positions a given window at the center of the monitor with cursor
fn position_window_at_the_center_of_the_monitor_with_cursor(window: &Window<Wry>) {
    if let Some(monitor) = get_monitor_with_cursor() {
        let display_size = monitor.size.to_logical::<f64>(monitor.scale_factor);
        let display_pos = monitor.position.to_logical::<f64>(monitor.scale_factor);

        let handle: id = window.ns_window().unwrap() as _;
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
}

/// Set the behaviors that makes the window appear on all workspaces
fn set_spotlight_window_collection_behavior(window: &Window<Wry>) {
    let handle: id = window.ns_window().unwrap() as _;
    unsafe {
        handle.setCollectionBehavior_(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenPrimary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle,
        );
    };
}

/// Set the window above menubar level
fn set_window_above_menubar(window: &Window<Wry>) {
    let handle: id = window.ns_window().unwrap() as _;
    unsafe { handle.setLevel_((NSMainMenuWindowLevel + 2).into()) };
}

struct Monitor {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub size: PhysicalSize<u32>,
    pub position: PhysicalPosition<i32>,
    pub scale_factor: f64,
}

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    pub fn NSMouseInRect(aPoint: NSPoint, aRect: NSRect, flipped: BOOL) -> BOOL;
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
            let screen_name = nsstring_to_string!(name);
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

pub fn get_frontmost_app_path() -> Option<String> {
    let shared_workspace: id = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    let frontmost_app: id = unsafe { msg_send![shared_workspace, frontmostApplication] };
    let bundle_url: id = unsafe { msg_send![frontmost_app, bundleURL] };
    let path: id = unsafe { msg_send![bundle_url, path] };
    nsstring_to_string!(path)
}
