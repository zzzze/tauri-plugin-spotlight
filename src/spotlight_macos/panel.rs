use core::fmt;

use bitflags::bitflags;
use objc_id::{Id, ShareId};
use cocoa::{
    appkit::{NSMainMenuWindowLevel, NSView, NSViewHeightSizable, NSViewWidthSizable, NSWindowCollectionBehavior},
    base::{id, nil, BOOL, YES}, foundation::NSRect,
};
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{self, Class, Object, Protocol, Sel},
    sel, sel_impl, Message,
};
use objc_foundation::INSObject;
use tauri::{Window, Wry};

extern "C" {
    pub fn object_setClass(obj: id, cls: id) -> id;
}

bitflags! {
    struct NSTrackingAreaOptions: u32 {
        const NSTrackingActiveAlways = 0x80;
        const NSTrackingMouseEnteredAndExited = 0x01;
        const NSTrackingMouseMoved = 0x02;
        const NSTrackingCursorUpdate = 0x04;
    }
}

#[allow(non_upper_case_globals)]
const NSWindowStyleMaskNonActivatingPanel: i32 = 1 << 7;

const CLS_NAME: &str = "RawNSPanel";

pub struct RawNSPanel;

impl RawNSPanel {
    fn get_class() -> &'static Class {
        Class::get(CLS_NAME).unwrap_or_else(Self::define_class)
    }

    fn define_class() -> &'static Class {
        let mut cls = ClassDecl::new(CLS_NAME, class!(NSPanel))
            .unwrap_or_else(|| panic!("Unable to register {} class", CLS_NAME));

        unsafe {
            cls.add_ivar::<BOOL>("_autoHide");

            cls.add_method(
                sel!(canBecomeKeyWindow),
                Self::can_become_key_window as extern "C" fn(&Object, Sel) -> BOOL,
            );

            cls.add_method(
                sel!(autoHide),
                Self::_get_auto_hide as extern "C" fn(&mut Object, Sel) -> BOOL,
            );

            cls.add_method(
                sel!(setAutoHide:),
                Self::_set_auto_hide as extern "C" fn(&mut Object, Sel, BOOL),
            );
        }

        cls.register()
    }

    extern "C" fn _get_auto_hide(this: &mut Object, _: Sel) -> BOOL {
        unsafe { *this.get_ivar("_autoHide") }
    }

    extern "C" fn _set_auto_hide(this: &mut Object, _: Sel, value: BOOL) {
        unsafe { this.set_ivar("_autoHide", value) };
    }

    /// Returns YES to ensure that RawNSPanel can become a key window
    extern "C" fn can_become_key_window(_: &Object, _: Sel) -> BOOL {
        YES
    }
}
unsafe impl Message for RawNSPanel {}

impl fmt::Debug for RawNSPanel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawNSPanel")
    }
}

impl RawNSPanel {
    pub(crate) fn show(&self) {
        self.make_first_responder(Some(self.content_view()));
        self.order_front_regardless();
        self.make_key_window();
    }

    pub(crate) fn is_visible(&self) -> bool {
        let flag: BOOL = unsafe { msg_send![self, isVisible] };
        flag == YES
    }

    pub(crate) fn make_key_window(&self) {
        let _: () = unsafe { msg_send![self, makeKeyWindow] };
    }

    pub(crate) fn order_front_regardless(&self) {
        let _: () = unsafe { msg_send![self, orderFrontRegardless] };
    }

    pub(crate) fn order_out(&self, sender: Option<id>) {
        let _: () = unsafe { msg_send![self, orderOut: sender.unwrap_or(nil)] };
    }

    pub(crate) fn content_view(&self) -> id {
        unsafe { msg_send![self, contentView] }
    }

    pub(crate) fn make_first_responder(&self, sender: Option<id>) {
        if let Some(responder) = sender {
            let _: () = unsafe { msg_send![self, makeFirstResponder: responder] };
        } else {
            let _: () = unsafe { msg_send![self, makeFirstResponder: self] };
        }
    }

    pub(crate) fn set_level(&self, level: i32) {
        let _: () = unsafe { msg_send![self, setLevel: level] };
    }

    pub(crate) fn set_auto_hide(&self, value: bool) {
        let _: () = unsafe { msg_send![self, setAutoHide: value] };
    }

    pub(crate) fn set_style_mask(&self, style_mask: i32) {
        let _: () = unsafe { msg_send![self, setStyleMask: style_mask] };
    }

    pub(crate) fn set_collection_behaviour(&self, behaviour: NSWindowCollectionBehavior) {
        let _: () = unsafe { msg_send![self, setCollectionBehavior: behaviour] };
    }

    fn set_delegate(&self, delegate: Option<Id<RawNSPanelDelegate>>) {
        if let Some(del) = delegate {
            let _: () = unsafe { msg_send![self, setDelegate: del] };
        } else {
            let _: () = unsafe { msg_send![self, setDelegate: self] };
        }
    }

    /// Create an NSPanel from Tauri's NSWindow
    fn from(ns_window: id) -> Id<Self> {
        let ns_panel: id = unsafe { msg_send![Self::class(), class] };
        unsafe {
            object_setClass(ns_window, ns_panel);
            Id::from_retained_ptr(ns_window as *mut Self)
        }
    }
}

impl INSObject for RawNSPanel {
    fn class() -> &'static runtime::Class {
        RawNSPanel::get_class()
    }
}

#[allow(dead_code)]
const DELEGATE_CLS_NAME: &str = "RawNSPanelDelegate";

#[allow(dead_code)]
struct RawNSPanelDelegate {}

impl RawNSPanelDelegate {
    #[allow(dead_code)]
    fn get_class() -> &'static Class {
        Class::get(DELEGATE_CLS_NAME).unwrap_or_else(Self::define_class)
    }

    #[allow(dead_code)]
    fn define_class() -> &'static Class {
        let mut cls = ClassDecl::new(DELEGATE_CLS_NAME, class!(NSObject))
            .unwrap_or_else(|| panic!("Unable to register {} class", DELEGATE_CLS_NAME));

        cls.add_protocol(
            Protocol::get("NSWindowDelegate").expect("Failed to get NSWindowDelegate protocol"),
        );

        unsafe {
            cls.add_ivar::<id>("panel");

            cls.add_method(
                sel!(setPanel:),
                Self::set_panel as extern "C" fn(&mut Object, Sel, id),
            );

            cls.add_method(
                sel!(windowDidBecomeKey:),
                Self::window_did_become_key as extern "C" fn(&Object, Sel, id),
            );

            cls.add_method(
                sel!(windowDidResignKey:),
                Self::window_did_resign_key as extern "C" fn(&Object, Sel, id),
            );
        }

        cls.register()
    }

    extern "C" fn set_panel(this: &mut Object, _: Sel, panel: id) {
        unsafe { this.set_ivar("panel", panel) };
    }

    extern "C" fn window_did_become_key(_: &Object, _: Sel, _: id) {}

    /// Hide panel when it's no longer the key window and auto hide is enabled
    extern "C" fn window_did_resign_key(this: &Object, _: Sel, _: id) {
        let panel: id = unsafe { *this.get_ivar("panel") };
        let auto_hide: BOOL = unsafe { msg_send![panel, autoHide] };

        if auto_hide == YES {
            let _: () = unsafe { msg_send![panel, orderOut: nil] };
        }
    }
}

unsafe impl Message for RawNSPanelDelegate {}

impl INSObject for RawNSPanelDelegate {
    fn class() -> &'static runtime::Class {
        Self::get_class()
    }
}

impl RawNSPanelDelegate {
    pub fn set_panel_(&self, panel: ShareId<RawNSPanel>) {
        let _: () = unsafe { msg_send![self, setPanel: panel] };
    }
}

pub(crate) fn create_spotlight_panel(window: &Window<Wry>) -> ShareId<RawNSPanel> {
    // Convert NSWindow Object to NSPanel
    let handle: id = window.ns_window().unwrap() as _;
    let panel = RawNSPanel::from(handle);
    let panel = panel.share();

    // Set panel above the main menu window level
    panel.set_level(NSMainMenuWindowLevel + 1);

    // Set panel to auto hide when it resigns key
    panel.set_auto_hide(true);

    // Ensure that the panel can display over the top of fullscreen apps
    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorTransient
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorMoveToActiveSpace
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
    );

    // Ensures panel does not activate
    panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel);

    // Setup delegate for an NSPanel to listen for window resign key and hide the panel
    let delegate = RawNSPanelDelegate::new();
    delegate.set_panel_(panel.clone());
    panel.set_delegate(Some(delegate));

    // On older macOS i.e on (12.3), hover detection is not working, see https://github.com/ahkohd/tauri-macos-spotlight-example/issues/14
    // To fix this, add a tracking view to the panel
    let view: id = panel.content_view();
    let bound: NSRect = unsafe { NSView::bounds(view) };
    let track_view: id = unsafe { msg_send![class!(NSTrackingArea), alloc] };
    let track_view: id = unsafe {
        msg_send![
            track_view,
            initWithRect: bound
            options: NSTrackingAreaOptions::NSTrackingActiveAlways
            | NSTrackingAreaOptions::NSTrackingMouseEnteredAndExited
            | NSTrackingAreaOptions::NSTrackingMouseMoved
            | NSTrackingAreaOptions::NSTrackingCursorUpdate
            owner: view
            userInfo: nil
        ]
    };
    let auto_resizing_mask = NSViewWidthSizable | NSViewHeightSizable;
    let () = unsafe { msg_send![view, setAutoresizingMask: auto_resizing_mask] };
    let () = unsafe { msg_send![view, addTrackingArea: track_view] };

    panel
}
