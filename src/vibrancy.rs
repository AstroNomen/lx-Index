#[cfg(target_os = "macos")]
#[link(name = "objc", kind = "dylib")]
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    static objc_msgSend: [u8; 0];
}

#[cfg(target_os = "macos")]
pub fn setup_macos_vibrancy() {
    unsafe {
        // Obtain NSApplication sharedApplication
        let ns_app_cls = objc_getClass(b"NSApplication\0".as_ptr() as *const _);
        if ns_app_cls.is_null() {
            return;
        }

        let sel_shared_app = sel_registerName(b"sharedApplication\0".as_ptr() as *const _);
        let msg_send_app: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void =
            std::mem::transmute(objc_msgSend.as_ptr());
        let shared_app = msg_send_app(ns_app_cls as _, sel_shared_app as _);
        if shared_app.is_null() {
            return;
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
            return;
        }

        let count = msg_send_usize(windows, sel_count as _);
        if count > 0 {
            println!("[Vibrancy] Detected {} window(s), applying macOS native titlebar integration & vibrancy", count);
        }
        for i in 0..count {
            let window = msg_send_at_idx(windows, sel_object_at_index as _, i);
            if !window.is_null() {
                apply_vibrancy_to_nswindow(window);
            }
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn apply_vibrancy_to_nswindow(window: *mut std::ffi::c_void) {
    let msg_send_bool: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, bool) =
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

    let msg_send_usize_get: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void) -> usize =
        std::mem::transmute(objc_msgSend.as_ptr());
    let msg_send_usize_set: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) =
        std::mem::transmute(objc_msgSend.as_ptr());

    // NSWindowStyleMaskFullSizeContentView = 1 << 15 = 32768
    let mut style_mask = msg_send_usize_get(window, sel_style_mask as _);
    style_mask |= 32768;
    msg_send_usize_set(window, sel_set_style_mask as _, style_mask);

    // titlebarAppearsTransparent = true
    msg_send_bool(window, sel_set_titlebar_transparent as _, true);

    // titleVisibility = NSWindowTitleHidden (1)
    msg_send_usize_set(window, sel_set_title_visibility as _, 1);

    // titlebarSeparatorStyle = NSTitlebarSeparatorStyleNone (0)
    msg_send_usize_set(window, sel_set_separator_style as _, 0);

    // movableByWindowBackground = true (window can be moved by dragging anywhere on top background)
    msg_send_bool(window, sel_set_movable as _, true);
}

#[cfg(not(target_os = "macos"))]
pub fn setup_macos_vibrancy() {
    // No-op on Windows/Linux
}
