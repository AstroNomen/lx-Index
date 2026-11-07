#[cfg(target_os = "macos")]
#[link(name = "objc", kind = "dylib")]
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn object_getClass(obj: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn objc_allocateClassPair(superclass: *mut std::ffi::c_void, name: *const std::ffi::c_char, extraBytes: usize) -> *mut std::ffi::c_void;
    fn objc_registerClassPair(cls: *mut std::ffi::c_void);
    fn class_addMethod(cls: *mut std::ffi::c_void, name: *mut std::ffi::c_void, imp: *const std::ffi::c_void, types: *const std::ffi::c_char) -> bool;
    fn class_replaceMethod(cls: *mut std::ffi::c_void, name: *mut std::ffi::c_void, imp: *const std::ffi::c_void, types: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn class_getMethodImplementation(cls: *mut std::ffi::c_void, name: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    static objc_msgSend: [u8; 0];
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct NSSize {
    width: f64,
    height: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

#[cfg(target_os = "macos")]
static mut GLOBAL_TOOLBAR: *mut std::ffi::c_void = std::ptr::null_mut();
#[cfg(target_os = "macos")]
static mut MAIN_WINDOW: *mut std::ffi::c_void = std::ptr::null_mut();
#[cfg(target_os = "macos")]
static mut ACTION_HANDLER: *mut std::ffi::c_void = std::ptr::null_mut();
#[cfg(target_os = "macos")]
static mut GLOBAL_MAIN_MENU: *mut std::ffi::c_void = std::ptr::null_mut();
#[cfg(target_os = "macos")]
static mut ORIGINAL_SEND_EVENT: Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void)> = None;
#[cfg(target_os = "macos")]
static NOTIFICATIONS_REGISTERED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(target_os = "macos")]
static SWIZZLED_SEND_EVENT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(target_os = "macos")]
extern "C" fn full_screen_presentation_options(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    _window: *mut std::ffi::c_void,
    proposed_options: usize,
) -> usize {
    // NSApplicationPresentationAutoHideToolbar = 1 << 11 = 2048
    // NSApplicationPresentationAutoHideMenuBar = 1 << 2 = 4
    // NSApplicationPresentationFullScreen = 1 << 10 = 1024
    proposed_options | (1 << 11) | (1 << 2) | (1 << 10)
}

#[cfg(target_os = "macos")]
unsafe fn get_or_create_toolbar() -> *mut std::ffi::c_void {
    if !GLOBAL_TOOLBAR.is_null() {
        return GLOBAL_TOOLBAR;
    }
    let ns_toolbar_cls = objc_getClass(b"NSToolbar\0".as_ptr() as *const _);
    let ns_str_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
    if ns_toolbar_cls.is_null() || ns_str_cls.is_null() {
        return std::ptr::null_mut();
    }
    let sel_alloc = sel_registerName(b"alloc\0".as_ptr() as *const _);
    let sel_init_with_id = sel_registerName(b"initWithIdentifier:\0".as_ptr() as *const _);
    let sel_str_with_utf8 = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
    let sel_set_shows_baseline = sel_registerName(b"setShowsBaselineSeparator:\0".as_ptr() as *const _);

    let msg_send_str: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_init_id: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
        std::mem::transmute(objc_msgSend.as_ptr());

    let tb_id = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"lxIndexToolbar\0".as_ptr() as _);
    let tb_alloc = msg_send_obj(ns_toolbar_cls as _, sel_alloc as _);
    let tb = msg_send_init_id(tb_alloc, sel_init_with_id as _, tb_id);
    if !tb.is_null() {
        msg_send_bool(tb, sel_set_shows_baseline as _, false);
        GLOBAL_TOOLBAR = tb;
    }
    GLOBAL_TOOLBAR
}

#[cfg(target_os = "macos")]
unsafe fn get_target_window_and_screen() -> Option<(*mut std::ffi::c_void, NSRect, NSRect)> {
    let ns_app_cls = objc_getClass(b"NSApplication\0".as_ptr() as *const _);
    if ns_app_cls.is_null() {
        return None;
    }
    let sel_shared_app = sel_registerName(b"sharedApplication\0".as_ptr() as *const _);
    let msg_send_obj: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let shared_app = msg_send_obj(ns_app_cls as _, sel_shared_app as _);
    if shared_app.is_null() {
        return None;
    }

    let msg_send_win: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let sel_key_window = sel_registerName(b"keyWindow\0".as_ptr() as *const _);
    let sel_main_window = sel_registerName(b"mainWindow\0".as_ptr() as *const _);
    let mut win = msg_send_win(shared_app, sel_key_window as _);
    if win.is_null() {
        win = msg_send_win(shared_app, sel_main_window as _);
    }
    if win.is_null() && !MAIN_WINDOW.is_null() {
        win = MAIN_WINDOW;
    }
    if win.is_null() {
        let sel_windows = sel_registerName(b"windows\0".as_ptr() as *const _);
        let sel_count = sel_registerName(b"count\0".as_ptr() as *const _);
        let sel_obj_idx = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const _);
        let wins = msg_send_win(shared_app, sel_windows as _);
        let msg_send_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> usize =
            std::mem::transmute(objc_msgSend.as_ptr());
        let count = if wins.is_null() { 0 } else { msg_send_usize(wins, sel_count as _) };
        if count > 0 {
            let msg_send_at_idx: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) -> *mut std::ffi::c_void =
                std::mem::transmute(objc_msgSend.as_ptr());
            win = msg_send_at_idx(wins, sel_obj_idx as _, 0);
        }
    }
    if win.is_null() {
        return None;
    }

    // If window is minimized, de-miniaturize it first
    let sel_is_min = sel_registerName(b"isMiniaturized\0".as_ptr() as *const _);
    let msg_send_bool_ret: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> bool =
        std::mem::transmute(objc_msgSend.as_ptr());
    if msg_send_bool_ret(win, sel_is_min as _) {
        let sel_demin = sel_registerName(b"deminiaturize:\0".as_ptr() as *const _);
        let msg_send_void_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_void_obj(win, sel_demin as _, std::ptr::null_mut());
    }

    // If in native fullscreen, exit fullscreen first so we can tile / resize
    let sel_style_mask = sel_registerName(b"styleMask\0".as_ptr() as *const _);
    let msg_send_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> usize =
        std::mem::transmute(objc_msgSend.as_ptr());
    let mask = msg_send_usize(win, sel_style_mask as _);
    if mask & (1 << 14) != 0 {
        let sel_toggle_fs = sel_registerName(b"toggleFullScreen:\0".as_ptr() as *const _);
        let msg_send_void_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_void_obj(win, sel_toggle_fs as _, std::ptr::null_mut());
    }

    let sel_screen = sel_registerName(b"screen\0".as_ptr() as *const _);
    let mut screen = msg_send_win(win, sel_screen as _);
    if screen.is_null() {
        let ns_screen_cls = objc_getClass(b"NSScreen\0".as_ptr() as *const _);
        let sel_main_screen = sel_registerName(b"mainScreen\0".as_ptr() as *const _);
        screen = msg_send_obj(ns_screen_cls as _, sel_main_screen as _);
    }
    if screen.is_null() {
        return None;
    }

    let sel_visible_frame = sel_registerName(b"visibleFrame\0".as_ptr() as *const _);
    let sel_frame = sel_registerName(b"frame\0".as_ptr() as *const _);
    let msg_send_rect: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> NSRect =
        std::mem::transmute(objc_msgSend.as_ptr());
    let vf = msg_send_rect(screen, sel_visible_frame as _);
    let wf = msg_send_rect(win, sel_frame as _);

    Some((win, vf, wf))
}

#[cfg(target_os = "macos")]
unsafe fn set_window_frame(win: *mut std::ffi::c_void, frame: NSRect) {
    let sel_set_frame = sel_registerName(b"setFrame:display:animate:\0".as_ptr() as *const _);
    let msg_send_set_frame: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, NSRect, bool, bool) =
        std::mem::transmute(objc_msgSend.as_ptr());
    msg_send_set_frame(win, sel_set_frame as _, frame, true, true);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_center(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, wf)) = get_target_window_and_screen() {
        let new_x = vf.origin.x + (vf.size.width - wf.size.width) / 2.0;
        let new_y = vf.origin.y + (vf.size.height - wf.size.height) / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: new_x, y: new_y },
            size: wf.size,
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_fill(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        set_window_frame(win, vf);
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_left(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_w = vf.size.width / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x, y: vf.origin.y },
            size: NSSize { width: half_w, height: vf.size.height },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_right(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_w = vf.size.width / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x + half_w, y: vf.origin.y },
            size: NSSize { width: half_w, height: vf.size.height },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_top(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_h = vf.size.height / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x, y: vf.origin.y + half_h },
            size: NSSize { width: vf.size.width, height: half_h },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_bottom(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_h = vf.size.height / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x, y: vf.origin.y },
            size: NSSize { width: vf.size.width, height: half_h },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_top_left(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_w = vf.size.width / 2.0;
        let half_h = vf.size.height / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x, y: vf.origin.y + half_h },
            size: NSSize { width: half_w, height: half_h },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_top_right(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_w = vf.size.width / 2.0;
        let half_h = vf.size.height / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x + half_w, y: vf.origin.y + half_h },
            size: NSSize { width: half_w, height: half_h },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_bottom_left(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_w = vf.size.width / 2.0;
        let half_h = vf.size.height / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x, y: vf.origin.y },
            size: NSSize { width: half_w, height: half_h },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_bottom_right(_this: *mut std::ffi::c_void, _cmd: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    if let Some((win, vf, _wf)) = get_target_window_and_screen() {
        let half_w = vf.size.width / 2.0;
        let half_h = vf.size.height / 2.0;
        set_window_frame(win, NSRect {
            origin: NSPoint { x: vf.origin.x + half_w, y: vf.origin.y },
            size: NSSize { width: half_w, height: half_h },
        });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn action_validate_menu_item(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    _item: *mut std::ffi::c_void,
) -> bool {
    true
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_window_will_enter_fullscreen(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    notification: *mut std::ffi::c_void,
) {
    let sel_object = sel_registerName(b"object\0".as_ptr() as *const _);
    let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let win = if !notification.is_null() {
        msg_send_obj(notification, sel_object as _)
    } else {
        MAIN_WINDOW
    };
    if !win.is_null() {
        // In fullscreen:
        // 1. Opaque frosted background on hover dropdown
        let sel_set_titlebar_transparent = sel_registerName(b"setTitlebarAppearsTransparent:\0".as_ptr() as *const _);
        let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_bool(win, sel_set_titlebar_transparent as _, false);

        // 2. Make title visible so macOS renders centered "lx Index" in the drop-down titlebar
        let sel_set_title_visibility = sel_registerName(b"setTitleVisibility:\0".as_ptr() as *const _);
        let msg_send_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_usize(win, sel_set_title_visibility as _, 0); // NSWindowTitleVisible = 0

        // 3. Compact titlebar style for native sleek fullscreen dropdown
        let sel_set_toolbar_style = sel_registerName(b"setToolbarStyle:\0".as_ptr() as *const _);
        msg_send_usize(win, sel_set_toolbar_style as _, 4); // NSWindowToolbarStyleUnifiedCompact = 4

        // Hide inactive overlay in fullscreen dropdown
        setup_or_update_traffic_lights_overlay(win, true);

        // Toolbar remains permanently attached and visible (no setVisible:false)
        // so window geometry is never collapsed, completely eliminating exit-fullscreen jumping!
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_window_will_exit_fullscreen(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    notification: *mut std::ffi::c_void,
) {
    let sel_object = sel_registerName(b"object\0".as_ptr() as *const _);
    let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let win = if !notification.is_null() {
        msg_send_obj(notification, sel_object as _)
    } else {
        MAIN_WINDOW
    };
    if !win.is_null() {
        // Restore transparent titlebar for windowed mode
        let sel_set_titlebar_transparent = sel_registerName(b"setTitlebarAppearsTransparent:\0".as_ptr() as *const _);
        let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_bool(win, sel_set_titlebar_transparent as _, true);

        // Hide system title text in windowed mode (custom Slint brand logo in left sidebar is used)
        let sel_set_title_visibility = sel_registerName(b"setTitleVisibility:\0".as_ptr() as *const _);
        let msg_send_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_usize(win, sel_set_title_visibility as _, 1); // NSWindowTitleHidden = 1

        let tb = get_or_create_toolbar();
        if !tb.is_null() {


            let sel_set_toolbar = sel_registerName(b"setToolbar:\0".as_ptr() as *const _);
            let msg_send_set_tb: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
                std::mem::transmute(objc_msgSend.as_ptr());
            msg_send_set_tb(win, sel_set_toolbar as _, tb);

            let sel_set_toolbar_style = sel_registerName(b"setToolbarStyle:\0".as_ptr() as *const _);
            msg_send_usize(win, sel_set_toolbar_style as _, 3); // Unified (3)
        }

        let sel_is_key = sel_registerName(b"isKeyWindow\0".as_ptr() as *const _);
        let msg_send_is_key: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> bool =
            std::mem::transmute(objc_msgSend.as_ptr());
        let is_key = msg_send_is_key(win, sel_is_key as _);
        setup_or_update_traffic_lights_overlay(win, is_key);
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_window_did_exit_fullscreen(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    notification: *mut std::ffi::c_void,
) {
    let sel_object = sel_registerName(b"object\0".as_ptr() as *const _);
    let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let win = if !notification.is_null() {
        msg_send_obj(notification, sel_object as _)
    } else {
        MAIN_WINDOW
    };
    if !win.is_null() {
        let sel_set_titlebar_transparent = sel_registerName(b"setTitlebarAppearsTransparent:\0".as_ptr() as *const _);
        let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_bool(win, sel_set_titlebar_transparent as _, true);

        let sel_set_title_visibility = sel_registerName(b"setTitleVisibility:\0".as_ptr() as *const _);
        let msg_send_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_usize(win, sel_set_title_visibility as _, 1); // NSWindowTitleHidden = 1

        let tb = get_or_create_toolbar();
        if !tb.is_null() {
            let sel_set_toolbar = sel_registerName(b"setToolbar:\0".as_ptr() as *const _);
            let msg_send_set_tb: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
                std::mem::transmute(objc_msgSend.as_ptr());
            msg_send_set_tb(win, sel_set_toolbar as _, tb);

            let sel_set_toolbar_style = sel_registerName(b"setToolbarStyle:\0".as_ptr() as *const _);
            msg_send_usize(win, sel_set_toolbar_style as _, 3); // Unified (3)
        }

        let sel_is_key = sel_registerName(b"isKeyWindow\0".as_ptr() as *const _);
        let msg_send_is_key: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> bool =
            std::mem::transmute(objc_msgSend.as_ptr());
        let is_key = msg_send_is_key(win, sel_is_key as _);
        setup_or_update_traffic_lights_overlay(win, is_key);
    }
}

#[cfg(target_os = "macos")]
static mut INACTIVE_OVERLAYS: [*mut std::ffi::c_void; 3] = [
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
];

#[cfg(target_os = "macos")]
unsafe fn setup_or_update_traffic_lights_overlay(win: *mut std::ffi::c_void, is_key: bool) {
    if win.is_null() {
        return;
    }
    let sel_btn = sel_registerName(b"standardWindowButton:\0".as_ptr() as *const _);
    let sel_responds = sel_registerName(b"respondsToSelector:\0".as_ptr() as *const _);
    let msg_send_responds: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *const std::ffi::c_void) -> bool =
        std::mem::transmute(objc_msgSend.as_ptr());

    // Safely resolve the target window instance and verify it responds to standardWindowButton:
    let target_win = if msg_send_responds(win, sel_responds as _, sel_btn as _) {
        win
    } else if !MAIN_WINDOW.is_null() && msg_send_responds(MAIN_WINDOW, sel_responds as _, sel_btn as _) {
        MAIN_WINDOW
    } else {
        return;
    };

    let msg_send_btn: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let sel_set_wants_layer = sel_registerName(b"setWantsLayer:\0".as_ptr() as *const _);
    let sel_layer = sel_registerName(b"layer\0".as_ptr() as *const _);
    let sel_set_frame = sel_registerName(b"setFrame:\0".as_ptr() as *const _);
    let sel_set_corner_radius = sel_registerName(b"setCornerRadius:\0".as_ptr() as *const _);
    let sel_set_bg_color = sel_registerName(b"setBackgroundColor:\0".as_ptr() as *const _);
    let sel_set_border_color = sel_registerName(b"setBorderColor:\0".as_ptr() as *const _);
    let sel_set_border_width = sel_registerName(b"setBorderWidth:\0".as_ptr() as *const _);
    let sel_set_hidden = sel_registerName(b"setHidden:\0".as_ptr() as *const _);
    let sel_add_sublayer = sel_registerName(b"addSublayer:\0".as_ptr() as *const _);

    let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_ptr: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_rect: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, NSRect) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_f64: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, f64) =
        std::mem::transmute(objc_msgSend.as_ptr());

    let calayer_cls = objc_getClass(b"CALayer\0".as_ptr() as *const _);
    let sel_layer_factory = sel_registerName(b"layer\0".as_ptr() as *const _);
    let ns_color_cls = objc_getClass(b"NSColor\0".as_ptr() as *const _);
    let sel_color_with_white = sel_registerName(b"colorWithCalibratedWhite:alpha:\0".as_ptr() as *const _);
    let sel_cg_color = sel_registerName(b"CGColor\0".as_ptr() as *const _);

    let msg_send_color: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, f64, f64) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());

    // Native macOS inactive traffic light colors: solid opaque grey circle
    let bg_color = msg_send_color(ns_color_cls as _, sel_color_with_white as _, 0.81, 1.0);
    let cg_bg = if !bg_color.is_null() { msg_send_obj(bg_color, sel_cg_color as _) } else { std::ptr::null_mut() };
    let border_color = msg_send_color(ns_color_cls as _, sel_color_with_white as _, 0.68, 1.0);
    let cg_border = if !border_color.is_null() { msg_send_obj(border_color, sel_cg_color as _) } else { std::ptr::null_mut() };

    for btn_id in 0..3 {
        let btn = msg_send_btn(target_win, sel_btn as _, btn_id);
        if btn.is_null() {
            continue;
        }
        msg_send_bool(btn, sel_set_wants_layer as _, true);
        let btn_layer = msg_send_obj(btn, sel_layer as _);
        if btn_layer.is_null() {
            continue;
        }

        let mut overlay = INACTIVE_OVERLAYS[btn_id];
        if overlay.is_null() && !calayer_cls.is_null() {
            overlay = msg_send_obj(calayer_cls as _, sel_layer_factory as _);
            if !overlay.is_null() {
                msg_send_rect(overlay, sel_set_frame as _, NSRect {
                    origin: NSPoint { x: 1.0, y: 2.0 },
                    size: NSSize { width: 12.0, height: 12.0 },
                });
                msg_send_f64(overlay, sel_set_corner_radius as _, 6.0);
                if !cg_bg.is_null() {
                    msg_send_ptr(overlay, sel_set_bg_color as _, cg_bg);
                }
                if !cg_border.is_null() {
                    msg_send_ptr(overlay, sel_set_border_color as _, cg_border);
                }
                msg_send_f64(overlay, sel_set_border_width as _, 0.5);
                msg_send_ptr(btn_layer, sel_add_sublayer as _, overlay);
                INACTIVE_OVERLAYS[btn_id] = overlay;
            }
        }

        if !overlay.is_null() {
            // is_key = true means active window -> overlay is hidden (true)
            // is_key = false means inactive window -> overlay is shown (false)
            msg_send_bool(overlay, sel_set_hidden as _, is_key);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_window_did_become_key(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    notification: *mut std::ffi::c_void,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let sel_object = sel_registerName(b"object\0".as_ptr() as *const _);
        let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let win = if !notification.is_null() {
            msg_send_obj(notification, sel_object as _)
        } else {
            MAIN_WINDOW
        };
        setup_or_update_traffic_lights_overlay(win, true);
    }));
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_window_did_resign_key(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    notification: *mut std::ffi::c_void,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let sel_object = sel_registerName(b"object\0".as_ptr() as *const _);
        let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let win = if !notification.is_null() {
            msg_send_obj(notification, sel_object as _)
        } else {
            MAIN_WINDOW
        };
        setup_or_update_traffic_lights_overlay(win, false);
    }));
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_app_did_become_active(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    _notification: *mut std::ffi::c_void,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if !MAIN_WINDOW.is_null() {
            setup_or_update_traffic_lights_overlay(MAIN_WINDOW, true);
        }
        // When switching back from another Space, un-minimizing, or gaining focus:
        // Re-assert our global main menu so macOS never drops or empties the menu bar!
        if !GLOBAL_MAIN_MENU.is_null() {
            let ns_app_cls = objc_getClass(b"NSApplication\0".as_ptr() as *const _);
            let sel_shared_app = sel_registerName(b"sharedApplication\0".as_ptr() as *const _);
            let msg_send_obj: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
                std::mem::transmute(objc_msgSend.as_ptr());
            let app = msg_send_obj(ns_app_cls as _, sel_shared_app as _);
            if !app.is_null() {
                let sel_set_main_menu = sel_registerName(b"setMainMenu:\0".as_ptr() as *const _);
                let msg_send_void_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
                    std::mem::transmute(objc_msgSend.as_ptr());
                msg_send_void_obj(app, sel_set_main_menu as _, GLOBAL_MAIN_MENU);
            }
        }
    }));
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_app_did_resign_active(
    _this: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
    _notification: *mut std::ffi::c_void,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if !MAIN_WINDOW.is_null() {
            setup_or_update_traffic_lights_overlay(MAIN_WINDOW, false);
        }
    }));
}

#[cfg(target_os = "macos")]
unsafe fn get_or_create_action_handler() -> *mut std::ffi::c_void {
    if !ACTION_HANDLER.is_null() {
        return ACTION_HANDLER;
    }
    let class_name = b"LxWindowActionHandler\0".as_ptr() as *const _;
    let mut cls = objc_getClass(class_name);
    if cls.is_null() {
        let superclass = objc_getClass(b"NSObject\0".as_ptr() as *const _);
        cls = objc_allocateClassPair(superclass, class_name, 0);
        if !cls.is_null() {
            let types = b"v@:@\0".as_ptr() as *const _;
            let methods: [(&[u8], unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void)); 17] = [
                (b"lxCenter:\0", action_center),
                (b"lxFill:\0", action_fill),
                (b"lxLeft:\0", action_left),
                (b"lxRight:\0", action_right),
                (b"lxTop:\0", action_top),
                (b"lxBottom:\0", action_bottom),
                (b"lxTopLeft:\0", action_top_left),
                (b"lxTopRight:\0", action_top_right),
                (b"lxBottomLeft:\0", action_bottom_left),
                (b"lxBottomRight:\0", action_bottom_right),
                (b"windowWillEnterFullScreen:\0", on_window_will_enter_fullscreen),
                (b"windowWillExitFullScreen:\0", on_window_will_exit_fullscreen),
                (b"windowDidExitFullScreen:\0", on_window_did_exit_fullscreen),
                (b"windowDidResignKey:\0", on_window_did_resign_key),
                (b"windowDidBecomeKey:\0", on_window_did_become_key),
                (b"appDidBecomeActive:\0", on_app_did_become_active),
                (b"appDidResignActive:\0", on_app_did_resign_active),
            ];
            for (sel_name, imp) in methods {
                let sel = sel_registerName(sel_name.as_ptr() as *const _);
                class_addMethod(cls, sel, imp as _, types);
            }
            let sel_validate = sel_registerName(b"validateMenuItem:\0".as_ptr() as *const _);
            class_addMethod(cls, sel_validate, action_validate_menu_item as _, b"B@:@\0".as_ptr() as *const _);

            objc_registerClassPair(cls);
        }
    }
    if !cls.is_null() {
        let sel_alloc = sel_registerName(b"alloc\0".as_ptr() as *const _);
        let sel_init = sel_registerName(b"init\0".as_ptr() as *const _);
        let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let alloc_inst = msg_send_obj(cls, sel_alloc as _);
        ACTION_HANDLER = msg_send_obj(alloc_inst, sel_init as _);
    }
    ACTION_HANDLER
}

#[cfg(target_os = "macos")]
unsafe fn register_fullscreen_notifications(window: *mut std::ffi::c_void) {
    if NOTIFICATIONS_REGISTERED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let handler = get_or_create_action_handler();
    if handler.is_null() {
        return;
    }
    let ns_notif_center_cls = objc_getClass(b"NSNotificationCenter\0".as_ptr() as *const _);
    let sel_default_center = sel_registerName(b"defaultCenter\0".as_ptr() as *const _);
    let msg_send_obj: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let center = msg_send_obj(ns_notif_center_cls as _, sel_default_center as _);
    if center.is_null() {
        return;
    }

    let ns_str_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
    let sel_str_with_utf8 = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
    let msg_send_str: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let sel_add_observer = sel_registerName(b"addObserver:selector:name:object:\0".as_ptr() as *const _);
    let msg_send_add_obs: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) =
        std::mem::transmute(objc_msgSend.as_ptr());

    // 1. Fullscreen enter and exit transitions
    let will_enter_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSWindowWillEnterFullScreenNotification\0".as_ptr() as _);
    let will_exit_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSWindowWillExitFullScreenNotification\0".as_ptr() as _);
    let did_exit_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSWindowDidExitFullScreenNotification\0".as_ptr() as _);

    let sel_will_enter = sel_registerName(b"windowWillEnterFullScreen:\0".as_ptr() as *const _);
    let sel_will_exit = sel_registerName(b"windowWillExitFullScreen:\0".as_ptr() as *const _);
    let sel_did_exit = sel_registerName(b"windowDidExitFullScreen:\0".as_ptr() as *const _);

    msg_send_add_obs(center, sel_add_observer as _, handler, sel_will_enter as _, will_enter_name, window);
    msg_send_add_obs(center, sel_add_observer as _, handler, sel_will_exit as _, will_exit_name, window);
    msg_send_add_obs(center, sel_add_observer as _, handler, sel_did_exit as _, did_exit_name, window);

    // 2. Lifecycle focus notifications: App activation, Window key, and Deminiaturize
    let sel_app_active = sel_registerName(b"appDidBecomeActive:\0".as_ptr() as *const _);
    let sel_app_resign = sel_registerName(b"appDidResignActive:\0".as_ptr() as *const _);
    let app_active_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSApplicationDidBecomeActiveNotification\0".as_ptr() as _);
    let app_resign_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSApplicationDidResignActiveNotification\0".as_ptr() as _);
    let win_key_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSWindowDidBecomeKeyNotification\0".as_ptr() as _);
    let win_resign_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSWindowDidResignKeyNotification\0".as_ptr() as _);
    let win_demin_name = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"NSWindowDidDeminiaturizeNotification\0".as_ptr() as _);

    let sel_key_change = sel_registerName(b"windowDidBecomeKey:\0".as_ptr() as *const _);
    let sel_resign_change = sel_registerName(b"windowDidResignKey:\0".as_ptr() as *const _);

    msg_send_add_obs(center, sel_add_observer as _, handler, sel_app_active as _, app_active_name, std::ptr::null_mut());
    msg_send_add_obs(center, sel_add_observer as _, handler, sel_app_resign as _, app_resign_name, std::ptr::null_mut());
    msg_send_add_obs(center, sel_add_observer as _, handler, sel_key_change as _, win_key_name, window);
    msg_send_add_obs(center, sel_add_observer as _, handler, sel_resign_change as _, win_resign_name, window);
    msg_send_add_obs(center, sel_add_observer as _, handler, sel_app_active as _, win_demin_name, window);
}

#[cfg(target_os = "macos")]
unsafe fn handle_titlebar_double_click(win: *mut std::ffi::c_void) {
    let ns_user_defaults_cls = objc_getClass(b"NSUserDefaults\0".as_ptr() as *const _);
    let sel_standard_ud = sel_registerName(b"standardUserDefaults\0".as_ptr() as *const _);
    let msg_send_obj: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let ud = msg_send_obj(ns_user_defaults_cls as _, sel_standard_ud as _);

    let mut should_minimize = true;
    if !ud.is_null() {
        let ns_str_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
        let sel_str_with_utf8 = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
        let msg_send_str: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let key_ns = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"AppleActionOnDoubleClick\0".as_ptr() as _);
        let sel_str_for_key = sel_registerName(b"stringForKey:\0".as_ptr() as *const _);
        let msg_send_val: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let val_ns = msg_send_val(ud, sel_str_for_key as _, key_ns);
        if !val_ns.is_null() {
            let sel_utf8_str = sel_registerName(b"UTF8String\0".as_ptr() as *const _);
            let msg_send_cstr: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *const std::ffi::c_char =
                std::mem::transmute(objc_msgSend.as_ptr());
            let cstr = msg_send_cstr(val_ns, sel_utf8_str as _);
            if !cstr.is_null() {
                let action_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy();
                if action_str == "Maximize" || action_str == "Zoom" {
                    should_minimize = false;
                } else if action_str == "None" {
                    return;
                }
            }
        }
    }

    let msg_send_void_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
        std::mem::transmute(objc_msgSend.as_ptr());
    if should_minimize {
        let sel_min = sel_registerName(b"performMiniaturize:\0".as_ptr() as *const _);
        msg_send_void_obj(win, sel_min as _, std::ptr::null_mut());
    } else {
        let sel_zoom = sel_registerName(b"performZoom:\0".as_ptr() as *const _);
        msg_send_void_obj(win, sel_zoom as _, std::ptr::null_mut());
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn custom_window_send_event(
    this: *mut std::ffi::c_void,
    cmd: *mut std::ffi::c_void,
    event: *mut std::ffi::c_void,
) {
    if !event.is_null() {
        let sel_type = sel_registerName(b"type\0".as_ptr() as *const _);
        let msg_send_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> usize =
            std::mem::transmute(objc_msgSend.as_ptr());
        let event_type = msg_send_usize(event, sel_type as _);
        // NSEventTypeLeftMouseUp = 2
        if event_type == 2 {
            let sel_click_count = sel_registerName(b"clickCount\0".as_ptr() as *const _);
            let msg_send_isize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> isize =
                std::mem::transmute(objc_msgSend.as_ptr());
            let click_count = msg_send_isize(event, sel_click_count as _);
            if click_count == 2 {
                let sel_loc = sel_registerName(b"locationInWindow\0".as_ptr() as *const _);
                let msg_send_point: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> NSPoint =
                    std::mem::transmute(objc_msgSend.as_ptr());
                let loc = msg_send_point(event, sel_loc as _);

                let sel_content_view = sel_registerName(b"contentView\0".as_ptr() as *const _);
                let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
                    std::mem::transmute(objc_msgSend.as_ptr());
                let cv = msg_send_obj(this, sel_content_view as _);
                if !cv.is_null() {
                    let sel_frame = sel_registerName(b"frame\0".as_ptr() as *const _);
                    let msg_send_rect: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> NSRect =
                        std::mem::transmute(objc_msgSend.as_ptr());
                    let cv_frame = msg_send_rect(cv, sel_frame as _);
                    // Top 52px area (Tall Titlebar Header)
                    if loc.y >= cv_frame.size.height - 52.0 && loc.x >= 0.0 && loc.x <= cv_frame.size.width {
                        handle_titlebar_double_click(this);
                    }
                }
            }
        }
    }
    if let Some(orig) = ORIGINAL_SEND_EVENT {
        orig(this, cmd, event);
    }
}

#[cfg(target_os = "macos")]
unsafe fn install_window_send_event_swizzle(window: *mut std::ffi::c_void) {
    if SWIZZLED_SEND_EVENT.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let win_cls = object_getClass(window);
    if win_cls.is_null() {
        return;
    }
    let sel_send_event = sel_registerName(b"sendEvent:\0".as_ptr() as *const _);
    let super_imp = class_getMethodImplementation(win_cls, sel_send_event as _);
    if !super_imp.is_null() {
        ORIGINAL_SEND_EVENT = Some(std::mem::transmute(super_imp));
    }
    class_replaceMethod(
        win_cls,
        sel_send_event as _,
        custom_window_send_event as _,
        b"v@:@\0".as_ptr() as *const _,
    );
}

#[cfg(target_os = "macos")]
pub fn setup_macos_vibrancy() -> bool {
    unsafe {
        let ns_app_cls = objc_getClass(b"NSApplication\0".as_ptr() as *const _);
        if ns_app_cls.is_null() {
            return false;
        }

        let sel_shared_app = sel_registerName(b"sharedApplication\0".as_ptr() as *const _);
        let msg_send_app: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let shared_app = msg_send_app(ns_app_cls as _, sel_shared_app as _);
        if shared_app.is_null() {
            return false;
        }

        let sel_windows = sel_registerName(b"windows\0".as_ptr() as *const _);
        let sel_count = sel_registerName(b"count\0".as_ptr() as *const _);
        let sel_object_at_index = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const _);

        let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> usize =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_at_idx: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());

        let windows = msg_send_obj(shared_app, sel_windows as _);
        let count = if windows.is_null() { 0 } else { msg_send_usize(windows, sel_count as _) };
        if count == 0 {
            let sel_key_window = sel_registerName(b"keyWindow\0".as_ptr() as *const _);
            let sel_main_window = sel_registerName(b"mainWindow\0".as_ptr() as *const _);
            let key_win = msg_send_obj(shared_app, sel_key_window as _);
            let main_win = msg_send_obj(shared_app, sel_main_window as _);
            let fallback_win = if !key_win.is_null() {
                key_win
            } else if !main_win.is_null() {
                main_win
            } else {
                std::ptr::null_mut()
            };

            if !fallback_win.is_null() {
                apply_vibrancy_to_nswindow(fallback_win);
                return true;
            }
            return false;
        }

        for i in 0..count {
            let window = msg_send_at_idx(windows, sel_object_at_index as _, i);
            if !window.is_null() {
                apply_vibrancy_to_nswindow(window);
            }
        }
        true
    }
}

#[cfg(target_os = "macos")]
unsafe fn apply_vibrancy_to_nswindow(window: *mut std::ffi::c_void) {
    MAIN_WINDOW = window;

    let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());

    // 1. Configure window sizing ceiling & min-size (prevents truncation / letterbox)
    let sel_set_max_size = sel_registerName(b"setMaxSize:\0".as_ptr() as *const _);
    let sel_set_min_size = sel_registerName(b"setMinSize:\0".as_ptr() as *const _);
    let sel_set_collection = sel_registerName(b"setCollectionBehavior:\0".as_ptr() as *const _);

    let msg_send_size: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, NSSize) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_u64: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, u64) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_usize_get: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> usize =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_usize_set: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) =
        std::mem::transmute(objc_msgSend.as_ptr());

    // Allow window to grow without limit up to 100,000 x 100,000
    msg_send_size(window, sel_set_max_size as _, NSSize { width: 100_000.0, height: 100_000.0 });
    msg_send_size(window, sel_set_min_size as _, NSSize { width: 960.0, height: 600.0 });

    // Preserve winit's default collectionBehavior (e.g. participatesInCycle) and add FullScreenPrimary & Tiling
    let sel_collection = sel_registerName(b"collectionBehavior\0".as_ptr() as *const _);
    let existing_cb = msg_send_usize_get(window, sel_collection as _);
    let new_cb = existing_cb | 128 | 2048 | 32;
    msg_send_u64(window, sel_set_collection as _, new_cb as u64);

    // 2. Seamless Titlebar Integration & Concentric Tall Titlebar
    let sel_style_mask = sel_registerName(b"styleMask\0".as_ptr() as *const _);
    let sel_set_style_mask = sel_registerName(b"setStyleMask:\0".as_ptr() as *const _);
    let sel_set_titlebar_transparent = sel_registerName(b"setTitlebarAppearsTransparent:\0".as_ptr() as *const _);
    let sel_set_title_visibility = sel_registerName(b"setTitleVisibility:\0".as_ptr() as *const _);
    let sel_set_separator_style = sel_registerName(b"setTitlebarSeparatorStyle:\0".as_ptr() as *const _);
    let sel_set_movable = sel_registerName(b"setMovableByWindowBackground:\0".as_ptr() as *const _);
    let sel_set_title = sel_registerName(b"setTitle:\0".as_ptr() as *const _);

    // NSWindowStyleMaskFullSizeContentView = 1 << 15 = 32768
    let mut style_mask = msg_send_usize_get(window, sel_style_mask as _);
    if style_mask & 32768 == 0 {
        style_mask |= 32768;
        msg_send_usize_set(window, sel_set_style_mask as _, style_mask);
    }

    // titlebarAppearsTransparent = true
    msg_send_bool(window, sel_set_titlebar_transparent as _, true);

    // titleVisibility = NSWindowTitleHidden (1)
    msg_send_usize_set(window, sel_set_title_visibility as _, 1);

    // Set window title to "lx Index" (used in fullscreen centered titlebar)
    let ns_str_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
    let sel_str_with_utf8 = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
    let msg_send_str: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let title_ns_str = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"lx Index\0".as_ptr() as _);
    let msg_send_ptr: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
        std::mem::transmute(objc_msgSend.as_ptr());
    msg_send_ptr(window, sel_set_title as _, title_ns_str);

    // titlebarSeparatorStyle = NSTitlebarSeparatorStyleNone (0)
    msg_send_usize_set(window, sel_set_separator_style as _, 0);

    // Window background color: use windowBackgroundColor (not clearColor) so inactive traffic lights composite as native disabled gray circles
    let ns_color_cls = objc_getClass(b"NSColor\0".as_ptr() as *const _);
    let sel_win_bg_color = sel_registerName(b"windowBackgroundColor\0".as_ptr() as *const _);
    let win_bg_color = msg_send_obj(ns_color_cls as _, sel_win_bg_color as _);
    let sel_set_bg_color = sel_registerName(b"setBackgroundColor:\0".as_ptr() as *const _);
    msg_send_ptr(window, sel_set_bg_color as _, win_bg_color);

    // 3. Tall Titlebar Unified Styling (Safari / Finder / System Settings concentric traffic lights)
    let toolbar = get_or_create_toolbar();
    let sel_set_toolbar = sel_registerName(b"setToolbar:\0".as_ptr() as *const _);
    let msg_send_set_tb: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let sel_set_toolbar_style = sel_registerName(b"setToolbarStyle:\0".as_ptr() as *const _);

    if !toolbar.is_null() {
        msg_send_usize_set(window, sel_set_toolbar_style as _, 3); // Unified (3)
        msg_send_set_tb(window, sel_set_toolbar as _, toolbar);
    }

    // Register observers for fullscreen enter/exit and app activation
    register_fullscreen_notifications(window);

    // Setup traffic lights active/inactive overlay matching window key state
    let sel_is_key = sel_registerName(b"isKeyWindow\0".as_ptr() as *const _);
    let msg_send_is_key: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> bool =
        std::mem::transmute(objc_msgSend.as_ptr());
    let is_key = msg_send_is_key(window, sel_is_key as _);
    setup_or_update_traffic_lights_overlay(window, is_key);

    // Ensure window delegate implements window:willUseFullScreenPresentationOptions: to autohide chrome in fullscreen
    let sel_delegate = sel_registerName(b"delegate\0".as_ptr() as *const _);
    let delegate = msg_send_obj(window, sel_delegate as _);
    if !delegate.is_null() {
        let delegate_cls = object_getClass(delegate);
        if !delegate_cls.is_null() {
            let sel_options = sel_registerName(b"window:willUseFullScreenPresentationOptions:\0".as_ptr() as *const _);
            let imp: unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void, usize) -> usize =
                full_screen_presentation_options;
            let types = b"Q@:@Q\0".as_ptr() as *const _;
            class_replaceMethod(delegate_cls, sel_options as _, imp as _, types);
        }
    }

    // Install event swizzling on the window class for native double-click titlebar minimize / zoom
    install_window_send_event_swizzle(window);

    // movableByWindowBackground = true (window can be moved by dragging anywhere on top background)
    msg_send_bool(window, sel_set_movable as _, true);

    // Explicitly update contentView frame to cover the entire window bounds
    let sel_content_view = sel_registerName(b"contentView\0".as_ptr() as *const _);
    let sel_frame = sel_registerName(b"frame\0".as_ptr() as *const _);
    let sel_set_frame = sel_registerName(b"setFrame:\0".as_ptr() as *const _);
    let content_view = msg_send_obj(window, sel_content_view as _);
    if !content_view.is_null() {
        let msg_send_rect: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> NSRect =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_set_rect: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, NSRect) =
            std::mem::transmute(objc_msgSend.as_ptr());
        let win_frame = msg_send_rect(window, sel_frame as _);
        let full_rect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: win_frame.size,
        };
        msg_send_set_rect(content_view, sel_set_frame as _, full_rect);
    }

    // Invalidate window shadow so WindowServer recalculates standard native window shadow
    let sel_set_has_shadow = sel_registerName(b"setHasShadow:\0".as_ptr() as *const _);
    let sel_invalidate_shadow = sel_registerName(b"invalidateShadow\0".as_ptr() as *const _);
    msg_send_bool(window, sel_set_has_shadow as _, true);
    let msg_send_void: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) =
        std::mem::transmute(objc_msgSend.as_ptr());
    msg_send_void(window, sel_invalidate_shadow as _);
}

#[cfg(not(target_os = "macos"))]
pub fn setup_macos_vibrancy() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn setup_macos_menu_bar(_lang: &str) -> bool {
    true
}

#[cfg(target_os = "macos")]
pub fn setup_macos_menu_bar(lang: &str) -> bool {
    unsafe {
        let ns_app_cls = objc_getClass(b"NSApplication\0".as_ptr() as *const _);
        if ns_app_cls.is_null() {
            return false;
        }

        let sel_shared_app = sel_registerName(b"sharedApplication\0".as_ptr() as *const _);
        let msg_send_app: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let shared_app = msg_send_app(ns_app_cls as _, sel_shared_app as _);
        if shared_app.is_null() {
            return false;
        }

        // Explicitly enforce regular application activation policy
        let sel_set_act_policy = sel_registerName(b"setActivationPolicy:\0".as_ptr() as *const _);
        let msg_send_bool_usize: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) -> bool =
            std::mem::transmute(objc_msgSend.as_ptr());
        msg_send_bool_usize(shared_app, sel_set_act_policy as _, 0); // NSApplicationActivationPolicyRegular = 0

        let ns_menu_cls = objc_getClass(b"NSMenu\0".as_ptr() as *const _);
        let ns_menu_item_cls = objc_getClass(b"NSMenuItem\0".as_ptr() as *const _);
        let ns_str_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
        if ns_menu_cls.is_null() || ns_menu_item_cls.is_null() || ns_str_cls.is_null() {
            return false;
        }

        let sel_alloc = sel_registerName(b"alloc\0".as_ptr() as *const _);
        let sel_init = sel_registerName(b"init\0".as_ptr() as *const _);
        let sel_init_with_title = sel_registerName(b"initWithTitle:\0".as_ptr() as *const _);
        let sel_str_with_utf8 = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
        let sel_add_item = sel_registerName(b"addItem:\0".as_ptr() as *const _);
        let sel_set_submenu = sel_registerName(b"setSubmenu:\0".as_ptr() as *const _);
        let sel_set_main_menu = sel_registerName(b"setMainMenu:\0".as_ptr() as *const _);
        let sel_separator_item = sel_registerName(b"separatorItem\0".as_ptr() as *const _);
        let sel_add_item_title_action_key = sel_registerName(b"addItemWithTitle:action:keyEquivalent:\0".as_ptr() as *const _);
        let sel_set_key_mask = sel_registerName(b"setKeyEquivalentModifierMask:\0".as_ptr() as *const _);
        let sel_set_target = sel_registerName(b"setTarget:\0".as_ptr() as *const _);
        let sel_set_title = sel_registerName(b"setTitle:\0".as_ptr() as *const _);
        let sel_set_autoenables = sel_registerName(b"setAutoenablesItems:\0".as_ptr() as *const _);

        let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_str: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_void_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_add_item: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_set_mask: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
            std::mem::transmute(objc_msgSend.as_ptr());

        let to_ns_str = |s: &str| -> *mut std::ffi::c_void {
            let mut c_bytes = s.as_bytes().to_vec();
            c_bytes.push(0);
            msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, c_bytes.as_ptr() as _)
        };

        let create_menu = |title: &str| -> *mut std::ffi::c_void {
            let m_alloc = msg_send_obj(ns_menu_cls as _, sel_alloc as _);
            let title_ns = to_ns_str(title);
            let msg_send_init_str: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void =
                std::mem::transmute(objc_msgSend.as_ptr());
            let m = msg_send_init_str(m_alloc, sel_init_with_title as _, title_ns);
            msg_send_bool(m, sel_set_autoenables as _, false);
            m
        };

        let create_item = |title: &str| -> *mut std::ffi::c_void {
            let item_alloc = msg_send_obj(ns_menu_item_cls as _, sel_alloc as _);
            let item = msg_send_obj(item_alloc, sel_init as _);
            if !title.is_empty() {
                msg_send_void_obj(item, sel_set_title as _, to_ns_str(title));
            }
            item
        };

        let add_item_to_menu = |menu: *mut std::ffi::c_void, title: &str, sel_name: Option<&[u8]>, key: &str, mask: Option<usize>| -> *mut std::ffi::c_void {
            let title_ns = to_ns_str(title);
            let key_ns = to_ns_str(key);
            let action_sel = match sel_name {
                Some(name) => sel_registerName(name.as_ptr() as *const _),
                None => std::ptr::null(),
            };
            let item = msg_send_add_item(menu, sel_add_item_title_action_key as _, title_ns, action_sel, key_ns);
            if let Some(m) = mask {
                if !item.is_null() {
                    msg_send_set_mask(item, sel_set_key_mask as _, m);
                }
            }
            item
        };

        let add_sep = |menu: *mut std::ffi::c_void| {
            let sep = msg_send_obj(ns_menu_item_cls as _, sel_separator_item as _);
            if !sep.is_null() {
                msg_send_void_obj(menu, sel_add_item as _, sep);
            }
        };

        let is_zh = lang == "zh";
        let action_handler = get_or_create_action_handler();

        // Create Main Menu Bar
        let main_menu = create_menu("MainMenu");

        // 1. App Submenu (lx-Index)
        let app_menu_item = create_item("lx-Index");
        let app_menu = create_menu("lx-Index");
        add_item_to_menu(app_menu, if is_zh { "关于 lx-Index" } else { "About lx-Index" }, Some(b"orderFrontStandardAboutPanel:\0"), "", None);
        add_sep(app_menu);
        add_item_to_menu(app_menu, if is_zh { "隐藏 lx-Index" } else { "Hide lx-Index" }, Some(b"hide:\0"), "h", None);
        add_item_to_menu(app_menu, if is_zh { "隐藏其他" } else { "Hide Others" }, Some(b"hideOtherApplications:\0"), "h", Some((1 << 20) | (1 << 19))); // Cmd + Opt + H
        add_item_to_menu(app_menu, if is_zh { "显示全部" } else { "Show All" }, Some(b"unhideAllApplications:\0"), "", None);
        add_sep(app_menu);
        add_item_to_menu(app_menu, if is_zh { "退出 lx-Index" } else { "Quit lx-Index" }, Some(b"terminate:\0"), "q", None);
        msg_send_void_obj(app_menu_item, sel_set_submenu as _, app_menu);
        msg_send_void_obj(main_menu, sel_add_item as _, app_menu_item);

        // 2. Edit Submenu
        let edit_title = if is_zh { "编辑" } else { "Edit" };
        let edit_menu_item = create_item(edit_title);
        let edit_menu = create_menu(edit_title);
        add_item_to_menu(edit_menu, if is_zh { "撤销" } else { "Undo" }, Some(b"undo:\0"), "z", None);
        add_item_to_menu(edit_menu, if is_zh { "重做" } else { "Redo" }, Some(b"redo:\0"), "Z", Some((1 << 20) | (1 << 17))); // Cmd + Shift + Z
        add_sep(edit_menu);
        add_item_to_menu(edit_menu, if is_zh { "剪切" } else { "Cut" }, Some(b"cut:\0"), "x", None);
        add_item_to_menu(edit_menu, if is_zh { "复制" } else { "Copy" }, Some(b"copy:\0"), "c", None);
        add_item_to_menu(edit_menu, if is_zh { "粘贴" } else { "Paste" }, Some(b"paste:\0"), "v", None);
        add_item_to_menu(edit_menu, if is_zh { "全选" } else { "Select All" }, Some(b"selectAll:\0"), "a", None);
        msg_send_void_obj(edit_menu_item, sel_set_submenu as _, edit_menu);
        msg_send_void_obj(main_menu, sel_add_item as _, edit_menu_item);

        // 3. Window Submenu (Authentic Apple macOS Window Management Menu)
        let win_title = if is_zh { "窗口" } else { "Window" };
        let window_menu_item = create_item(win_title);
        let window_menu = create_menu(win_title);

        // 最小化 (Minimize - Cmd+M)
        add_item_to_menu(window_menu, if is_zh { "最小化" } else { "Minimize" }, Some(b"performMiniaturize:\0"), "m", None);
        // 缩放 (Zoom)
        add_item_to_menu(window_menu, if is_zh { "缩放" } else { "Zoom" }, Some(b"performZoom:\0"), "", None);

        // 填充屏幕 (Fill Screen)
        let item_fill = add_item_to_menu(window_menu, if is_zh { "填充屏幕" } else { "Fill" }, Some(b"lxFill:\0"), "", None);
        if !item_fill.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_fill, sel_set_target as _, action_handler);
        }

        // 居中窗口 (Center Window)
        let item_center = add_item_to_menu(window_menu, if is_zh { "居中窗口" } else { "Center" }, Some(b"lxCenter:\0"), "", None);
        if !item_center.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_center, sel_set_target as _, action_handler);
        }

        add_sep(window_menu);

        // 移动与调整大小 (Move & Resize Submenu - macOS Sequoia Window Tiling)
        let tile_title = if is_zh { "移动与调整大小" } else { "Move & Resize" };
        let tile_menu_item = create_item(tile_title);
        let tile_menu = create_menu(tile_title);

        let item_left = add_item_to_menu(tile_menu, if is_zh { "平铺到屏幕左侧" } else { "Left" }, Some(b"lxLeft:\0"), "", None);
        if !item_left.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_left, sel_set_target as _, action_handler);
        }

        let item_right = add_item_to_menu(tile_menu, if is_zh { "平铺到屏幕右侧" } else { "Right" }, Some(b"lxRight:\0"), "", None);
        if !item_right.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_right, sel_set_target as _, action_handler);
        }

        let item_top = add_item_to_menu(tile_menu, if is_zh { "平铺到屏幕上方" } else { "Top" }, Some(b"lxTop:\0"), "", None);
        if !item_top.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_top, sel_set_target as _, action_handler);
        }

        let item_bottom = add_item_to_menu(tile_menu, if is_zh { "平铺到屏幕下方" } else { "Bottom" }, Some(b"lxBottom:\0"), "", None);
        if !item_bottom.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_bottom, sel_set_target as _, action_handler);
        }

        add_sep(tile_menu);

        let item_tl = add_item_to_menu(tile_menu, if is_zh { "左上分屏" } else { "Top Left" }, Some(b"lxTopLeft:\0"), "", None);
        if !item_tl.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_tl, sel_set_target as _, action_handler);
        }

        let item_tr = add_item_to_menu(tile_menu, if is_zh { "右上分屏" } else { "Top Right" }, Some(b"lxTopRight:\0"), "", None);
        if !item_tr.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_tr, sel_set_target as _, action_handler);
        }

        let item_bl = add_item_to_menu(tile_menu, if is_zh { "左下分屏" } else { "Bottom Left" }, Some(b"lxBottomLeft:\0"), "", None);
        if !item_bl.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_bl, sel_set_target as _, action_handler);
        }

        let item_br = add_item_to_menu(tile_menu, if is_zh { "右下分屏" } else { "Bottom Right" }, Some(b"lxBottomRight:\0"), "", None);
        if !item_br.is_null() && !action_handler.is_null() {
            msg_send_void_obj(item_br, sel_set_target as _, action_handler);
        }

        msg_send_void_obj(tile_menu_item, sel_set_submenu as _, tile_menu);
        msg_send_void_obj(window_menu, sel_add_item as _, tile_menu_item);

        add_sep(window_menu);

        // 进入全屏幕 (Enter Full Screen - Ctrl+Cmd+F)
        add_item_to_menu(window_menu, if is_zh { "进入全屏幕" } else { "Toggle Full Screen" }, Some(b"toggleFullScreen:\0"), "f", Some((1 << 20) | (1 << 18)));

        add_sep(window_menu);

        // 关闭窗口 (Close Window - Cmd+W)
        add_item_to_menu(window_menu, if is_zh { "关闭窗口" } else { "Close Window" }, Some(b"performClose:\0"), "w", None);

        add_sep(window_menu);

        // 前置全部窗口 (Bring All to Front)
        add_item_to_menu(window_menu, if is_zh { "前置全部窗口" } else { "Bring All to Front" }, Some(b"arrangeInFront:\0"), "", None);

        msg_send_void_obj(window_menu_item, sel_set_submenu as _, window_menu);
        msg_send_void_obj(main_menu, sel_add_item as _, window_menu_item);

        GLOBAL_MAIN_MENU = main_menu;
        msg_send_void_obj(shared_app, sel_set_main_menu as _, main_menu);
        true
    }
}
