#[cfg(target_os = "macos")]
#[link(name = "objc", kind = "dylib")]
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    static objc_msgSend: [u8; 0];
}

#[cfg(target_os = "macos")]
pub fn setup_macos_vibrancy() -> bool {
    unsafe {
        // Obtain NSApplication sharedApplication
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
        if windows.is_null() {
            return false;
        }

        let count = msg_send_usize(windows, sel_count as _);
        if count == 0 {
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
    let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_obj: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());

    // 1. Configure window sizing ceiling & min-size (prevents truncation / letterbox)
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NSSize {
        width: f64,
        height: f64,
    }
    let sel_set_max_size = sel_registerName(b"setMaxSize:\0".as_ptr() as *const _);
    let sel_set_min_size = sel_registerName(b"setMinSize:\0".as_ptr() as *const _);
    let sel_set_collection = sel_registerName(b"setCollectionBehavior:\0".as_ptr() as *const _);

    let msg_send_size: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, NSSize) =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_u64: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, u64) =
        std::mem::transmute(objc_msgSend.as_ptr());

    // Allow window to grow without limit up to 100,000 x 100,000 (eliminates fullscreen letterbox/black borders)
    msg_send_size(window, sel_set_max_size as _, NSSize { width: 100_000.0, height: 100_000.0 });
    msg_send_size(window, sel_set_min_size as _, NSSize { width: 960.0, height: 600.0 });

    // Enable native fullscreen: NSWindowCollectionBehaviorFullScreenPrimary (1 << 7 = 128) | NSWindowCollectionBehaviorManaged (1 << 2 = 4)
    msg_send_u64(window, sel_set_collection as _, 128 | 4);

    // 2. Seamless Titlebar Integration: Make titlebar transparent & extend content under titlebar
    let sel_style_mask = sel_registerName(b"styleMask\0".as_ptr() as *const _);
    let sel_set_style_mask = sel_registerName(b"setStyleMask:\0".as_ptr() as *const _);
    let sel_set_titlebar_transparent = sel_registerName(b"setTitlebarAppearsTransparent:\0".as_ptr() as *const _);
    let sel_set_title_visibility = sel_registerName(b"setTitleVisibility:\0".as_ptr() as *const _);
    let sel_set_separator_style = sel_registerName(b"setTitlebarSeparatorStyle:\0".as_ptr() as *const _);
    let sel_set_movable = sel_registerName(b"setMovableByWindowBackground:\0".as_ptr() as *const _);
    let sel_set_title = sel_registerName(b"setTitle:\0".as_ptr() as *const _);

    let msg_send_usize_get: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> usize =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_usize_set: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) =
        std::mem::transmute(objc_msgSend.as_ptr());

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

    // Empty title so no system text is rendered in title area
    let ns_str_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
    let sel_str_with_utf8 = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
    let msg_send_str: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
        std::mem::transmute(objc_msgSend.as_ptr());
    let empty_ns_str = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"\0".as_ptr() as _);
    let msg_send_ptr: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
        std::mem::transmute(objc_msgSend.as_ptr());
    msg_send_ptr(window, sel_set_title as _, empty_ns_str);

    // titlebarSeparatorStyle = NSTitlebarSeparatorStyleNone (0)
    msg_send_usize_set(window, sel_set_separator_style as _, 0);

    // setToolbarStyle: NSWindowToolbarStyleUnified (1)
    let sel_set_toolbar_style = sel_registerName(b"setToolbarStyle:\0".as_ptr() as *const _);
    msg_send_usize_set(window, sel_set_toolbar_style as _, 1);

    // Attach an empty NSToolbar to switch window into unified toolbar mode.
    // In unified toolbar mode, macOS automatically places the traffic lights at (20pt, 20pt) from top-left,
    // making them concentric with the window's rounded corner radius (R=20pt) and aligned with the HUD bar.
    let sel_toolbar = sel_registerName(b"toolbar\0".as_ptr() as *const _);
    let existing_toolbar = msg_send_obj(window, sel_toolbar as _);
    if existing_toolbar.is_null() {
        let ns_toolbar_cls = objc_getClass(b"NSToolbar\0".as_ptr() as *const _);
        if !ns_toolbar_cls.is_null() {
            let sel_alloc = sel_registerName(b"alloc\0".as_ptr() as *const _);
            let sel_init_with_id = sel_registerName(b"initWithIdentifier:\0".as_ptr() as *const _);
            let sel_set_toolbar = sel_registerName(b"setToolbar:\0".as_ptr() as *const _);
            let sel_set_shows_baseline = sel_registerName(b"setShowsBaselineSeparator:\0".as_ptr() as *const _);

            let tb_id = msg_send_str(ns_str_cls as _, sel_str_with_utf8 as _, b"lxIndexToolbar\0".as_ptr() as _);
            let tb_alloc = msg_send_obj(ns_toolbar_cls as _, sel_alloc as _);
            let msg_send_init_id: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void =
                std::mem::transmute(objc_msgSend.as_ptr());
            let toolbar = msg_send_init_id(tb_alloc, sel_init_with_id as _, tb_id);
            if !toolbar.is_null() {
                msg_send_bool(toolbar, sel_set_shows_baseline as _, false);
                let msg_send_set_tb: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, *mut std::ffi::c_void) =
                    std::mem::transmute(objc_msgSend.as_ptr());
                msg_send_set_tb(window, sel_set_toolbar as _, toolbar);
            }
        }
    }

    // movableByWindowBackground = true (window can be moved by dragging anywhere on top background)
    msg_send_bool(window, sel_set_movable as _, true);

    // Explicitly update contentView frame to cover the entire window bounds
    let sel_content_view = sel_registerName(b"contentView\0".as_ptr() as *const _);
    let sel_frame = sel_registerName(b"frame\0".as_ptr() as *const _);
    let sel_set_frame = sel_registerName(b"setFrame:\0".as_ptr() as *const _);
    let content_view = msg_send_obj(window, sel_content_view as _);
    if !content_view.is_null() {
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct NSRect {
            x: f64,
            y: f64,
            width: f64,
            height: f64,
        }
        let msg_send_rect: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> NSRect =
            std::mem::transmute(objc_msgSend.as_ptr());
        let msg_send_set_rect: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, NSRect) =
            std::mem::transmute(objc_msgSend.as_ptr());
        let win_frame = msg_send_rect(window, sel_frame as _);
        let full_rect = NSRect {
            x: 0.0,
            y: 0.0,
            width: win_frame.width,
            height: win_frame.height,
        };
        msg_send_set_rect(content_view, sel_set_frame as _, full_rect);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn setup_macos_vibrancy() -> bool {
    true
}
