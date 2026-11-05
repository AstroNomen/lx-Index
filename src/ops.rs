use std::path::{Path, PathBuf};
use crate::frontmatter::{extract_bibtex_block, generate_fallback_bibtex, parse_note_content};
use crate::models::PaperMetadata;

/// Reveals the given file or directory in the OS file manager.
/// - macOS: `open -R <path>` (Reveals item in Finder)
/// - Windows: `explorer /select,"<path>"` (Selects item in File Explorer)
/// - Other: opens containing folder
pub fn reveal_in_file_manager(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path does not exist: {}", path.display()),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,\"{}\"", path.display()))
            .spawn()?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        if let Some(parent) = path.parent() {
            let _ = open::that(parent);
        } else {
            let _ = open::that(path);
        }
    }

    Ok(())
}

/// Native OS folder selection dialog without extra dependencies.
pub fn pick_folder() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("POSIX path of (choose folder with prompt \"Select lx Index Literature Vault\")")
            .output()
            .ok()?;
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        rfd::FileDialog::new()
            .set_title("Select lx Index Literature Vault")
            .pick_folder()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        rfd::FileDialog::new()
            .set_title("Select lx Index Literature Vault")
            .pick_folder()
    }
}

/// Copies arbitrary text to the system clipboard using arboard.
pub fn copy_text_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}

/// Copies a citekey (e.g., `@Bollani2026`) to the system clipboard.
pub fn copy_citekey_to_clipboard(citekey: &str) -> Result<(), Box<dyn std::error::Error>> {
    let formatted = if citekey.starts_with('@') {
        citekey.to_string()
    } else {
        format!("@{}", citekey)
    };
    copy_text_to_clipboard(&formatted)
}

/// Extracts a BibTeX block from note content or generates one from frontmatter metadata.
pub fn extract_or_generate_bibtex(content: &str, fallback_meta: Option<&PaperMetadata>) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(bibtex) = extract_bibtex_block(content) {
        return Ok(bibtex);
    }

    if let Ok((meta, _)) = parse_note_content(content) {
        return Ok(generate_fallback_bibtex(&meta));
    }

    if let Some(meta) = fallback_meta {
        return Ok(generate_fallback_bibtex(meta));
    }

    Err("Could not extract or generate BibTeX citation".into())
}

/// Copies the paper's BibTeX citation to the system clipboard from its companion note file.
pub fn copy_bibtex_to_clipboard(note_path: &Path, fallback_meta: Option<&PaperMetadata>) -> Result<(), Box<dyn std::error::Error>> {
    let content = if note_path.exists() {
        std::fs::read_to_string(note_path)?
    } else if let Some(meta) = fallback_meta {
        let bibtex = generate_fallback_bibtex(meta);
        return copy_text_to_clipboard(&bibtex);
    } else {
        return Err(format!("Note file does not exist: {}", note_path.display()).into());
    };

    let bibtex = extract_or_generate_bibtex(&content, fallback_meta)?;
    copy_text_to_clipboard(&bibtex)?;
    Ok(())
}

/// Safely moves the PDF and/or companion note to the OS Trash / Recycle Bin.
pub fn safe_delete_paper(
    pdf_path: Option<&Path>,
    note_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut deleted_any = false;

    if let Some(path) = pdf_path {
        if path.exists() {
            trash::delete(path)?;
            deleted_any = true;
        }
    }

    if let Some(path) = note_path {
        if path.exists() {
            trash::delete(path)?;
            deleted_any = true;
        }
    }

    if !deleted_any {
        return Err("No existing files were provided to delete".into());
    }

    Ok(())
}

pub fn window_drag() {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetActiveWindow() -> *mut std::ffi::c_void;
            fn ReleaseCapture() -> i32;
            fn SendMessageW(hwnd: *mut std::ffi::c_void, msg: u32, wparam: usize, lparam: isize) -> isize;
        }
        unsafe {
            let hwnd = GetActiveWindow();
            if !hwnd.is_null() {
                ReleaseCapture();
                SendMessageW(hwnd, 0x00A1, 2, 0);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod win_frameless {
    use std::ffi::c_void;

    #[repr(C)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct MONITORINFO {
        pub cbSize: u32,
        pub rcMonitor: RECT,
        pub rcWork: RECT,
        pub dwFlags: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct MARGINS {
        pub cxLeftWidth: i32,
        pub cxRightWidth: i32,
        pub cyTopHeight: i32,
        pub cyBottomHeight: i32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct NCCALCSIZE_PARAMS {
        pub rgrc: [RECT; 3],
        pub lppos: *mut c_void,
    }

    extern "system" {
        pub fn CallWindowProcW(prev_proc: isize, hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) -> isize;
        pub fn GetWindowRect(hwnd: *mut c_void, rect: *mut RECT) -> i32;
        pub fn IsZoomed(hwnd: *mut c_void) -> i32;
        pub fn GetSystemMetrics(n_index: i32) -> i32;
        pub fn MonitorFromWindow(hwnd: *mut c_void, dwFlags: u32) -> *mut c_void;
        pub fn GetMonitorInfoW(hMonitor: *mut c_void, lpmi: *mut MONITORINFO) -> i32;
        pub fn LoadLibraryA(lpLibFileName: *const u8) -> *mut c_void;
        pub fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
    }

    pub const WM_NCCALCSIZE: u32 = 0x0083;
    pub const WM_NCHITTEST: u32 = 0x0084;
    pub const WM_MOUSEACTIVATE: u32 = 0x0021;
    pub const WM_SIZE: u32 = 0x0005;

    pub const HTCLIENT: isize = 1;
    pub const HTCAPTION: isize = 2;
    pub const HTLEFT: isize = 10;
    pub const HTRIGHT: isize = 11;
    pub const HTTOP: isize = 12;
    pub const HTTOPLEFT: isize = 13;
    pub const HTTOPRIGHT: isize = 14;
    pub const HTBOTTOM: isize = 15;
    pub const HTBOTTOMLEFT: isize = 16;
    pub const HTBOTTOMRIGHT: isize = 17;

    pub const MA_ACTIVATE: isize = 1;

    pub static mut PREV_WNDPROC: isize = 0;

    pub unsafe extern "system" fn frameless_wndproc(
        hwnd: *mut c_void,
        msg: u32,
        wparam: usize,
        lparam: isize,
    ) -> isize {
        match msg {
            WM_NCCALCSIZE => {
                if wparam != 0 {
                    if IsZoomed(hwnd) != 0 {
                        // Monitor-based work area alignment (Chromium/WPF standard frameless method)
                        // This perfectly matches screen work area with 0 extra OS borders or white lines
                        let hmon = MonitorFromWindow(hwnd, 2 /* MONITOR_DEFAULTTONEAREST */);
                        if !hmon.is_null() {
                            let mut mi = MONITORINFO {
                                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                                rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                                rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                                dwFlags: 0,
                            };
                            if GetMonitorInfoW(hmon, &mut mi) != 0 {
                                let params = &mut *(lparam as *mut NCCALCSIZE_PARAMS);
                                params.rgrc[0] = mi.rcWork;
                                return 0;
                            }
                        }
                        // Fallback: strip frames without calling default proc
                        let params = &mut *(lparam as *mut NCCALCSIZE_PARAMS);
                        let frame_x = GetSystemMetrics(32 /* SM_CXFRAME */) + GetSystemMetrics(92 /* SM_CXPADDEDBORDER */);
                        let frame_y = GetSystemMetrics(33 /* SM_CYFRAME */) + GetSystemMetrics(92 /* SM_CXPADDEDBORDER */);
                        params.rgrc[0].left += frame_x;
                        params.rgrc[0].right -= frame_x;
                        params.rgrc[0].top += frame_y;
                        params.rgrc[0].bottom -= frame_y;
                        return 0;
                    }
                    // When not maximized, return 0 to occupy 100% of window (no OS titlebar)
                    return 0;
                }
                CallWindowProcW(PREV_WNDPROC, hwnd, msg, wparam, lparam)
            }
            WM_SIZE => {
                let is_max = IsZoomed(hwnd) != 0;
                let margins = if is_max {
                    MARGINS { cxLeftWidth: 0, cxRightWidth: 0, cyTopHeight: 0, cyBottomHeight: 0 }
                } else {
                    MARGINS { cxLeftWidth: 1, cxRightWidth: 1, cyTopHeight: 1, cyBottomHeight: 1 }
                };
                let dwm = LoadLibraryA(b"dwmapi.dll\0".as_ptr());
                if !dwm.is_null() {
                    let extend_proc = GetProcAddress(dwm, b"DwmExtendFrameIntoClientArea\0".as_ptr());
                    if !extend_proc.is_null() {
                        type DwmExtendFrameFn = unsafe extern "system" fn(*mut c_void, *const MARGINS) -> i32;
                        let extend_frame: DwmExtendFrameFn = std::mem::transmute(extend_proc);
                        extend_frame(hwnd, &margins);
                    }
                }
                CallWindowProcW(PREV_WNDPROC, hwnd, msg, wparam, lparam)
            }
            WM_NCHITTEST => {
                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                GetWindowRect(hwnd, &mut rect);

                let x = (lparam as i32 & 0xFFFF) as i16 as i32;
                let y = ((lparam as i32 >> 16) & 0xFFFF) as i16 as i32;
                let win_w = rect.right - rect.left;
                let win_h = rect.bottom - rect.top;
                let rx = x - rect.left;
                let ry = y - rect.top;

                let is_max = IsZoomed(hwnd) != 0;

                // Native border resize hit testing (only when not maximized)
                if !is_max {
                    const B: i32 = 8;
                    if rx < B && ry < B { return HTTOPLEFT; }
                    if rx > win_w - B && ry < B { return HTTOPRIGHT; }
                    if rx < B && ry > win_h - B { return HTBOTTOMLEFT; }
                    if rx > win_w - B && ry > win_h - B { return HTBOTTOMRIGHT; }
                    if rx < B { return HTLEFT; }
                    if rx > win_w - B { return HTRIGHT; }
                    if ry < B { return HTTOP; }
                    if ry > win_h - B { return HTBOTTOM; }
                }

                // Titlebar drag area (top 42px, excluding window control buttons on top right)
                // The 3 buttons take 46px * 3 = 138px on the right
                if ry >= 0 && ry < 42 {
                    if rx < win_w - 140 {
                        return HTCAPTION;
                    } else {
                        return HTCLIENT;
                    }
                }

                HTCLIENT
            }
            WM_MOUSEACTIVATE => {
                // Ensure immediate click response on first click even after focus loss
                MA_ACTIVATE
            }
            _ => CallWindowProcW(PREV_WNDPROC, hwnd, msg, wparam, lparam),
        }
    }
}

/// Configures native Windows DWM drop shadow, resizable borders and window styling for frameless mode without ugly borders.
pub fn setup_windows_window() {
    #[cfg(target_os = "windows")]
    {
        use win_frameless::MARGINS;
        extern "system" {
            fn GetActiveWindow() -> *mut std::ffi::c_void;
            fn GetWindowLongPtrW(hwnd: *mut std::ffi::c_void, index: i32) -> isize;
            fn SetWindowLongPtrW(hwnd: *mut std::ffi::c_void, index: i32, new_long: isize) -> isize;
            fn SetWindowPos(hwnd: *mut std::ffi::c_void, insert_after: *mut std::ffi::c_void, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
            fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
            fn GetProcAddress(hModule: *mut std::ffi::c_void, lpProcName: *const u8) -> *mut std::ffi::c_void;
        }

        unsafe {
            let hwnd = GetActiveWindow();
            if hwnd.is_null() {
                return;
            }

            // 1. Ensure window has standard resizable borders and minimize/maximize styles
            const GWL_STYLE: i32 = -16;
            const GWLP_WNDPROC: i32 = -4;
            const WS_THICKFRAME: isize = 0x00040000;
            const WS_MINIMIZEBOX: isize = 0x00020000;
            const WS_MAXIMIZEBOX: isize = 0x00010000;

            let cur_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let target_style = cur_style | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
            SetWindowLongPtrW(hwnd, GWL_STYLE, target_style);

            // 2. Subclass window procedure to handle WM_NCCALCSIZE, WM_NCHITTEST, and WM_MOUSEACTIVATE
            if win_frameless::PREV_WNDPROC == 0 {
                let prev = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, win_frameless::frameless_wndproc as *const () as isize);
                win_frameless::PREV_WNDPROC = prev;
            }

            // 3. Trigger frame update
            const SWP_NOSIZE: u32 = 0x0001;
            const SWP_NOMOVE: u32 = 0x0002;
            const SWP_NOZORDER: u32 = 0x0004;
            const SWP_FRAMECHANGED: u32 = 0x0040;
            SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED);

            // 4. DWM APIs (dynamically loaded from dwmapi.dll)
            let dwm = LoadLibraryA(b"dwmapi.dll\0".as_ptr());
            if !dwm.is_null() {
                type DwmExtendFrameFn = unsafe extern "system" fn(*mut std::ffi::c_void, *const MARGINS) -> i32;
                type DwmSetAttrFn = unsafe extern "system" fn(*mut std::ffi::c_void, u32, *const std::ffi::c_void, u32) -> i32;

                let extend_proc = GetProcAddress(dwm, b"DwmExtendFrameIntoClientArea\0".as_ptr());
                if !extend_proc.is_null() {
                    let extend_frame: DwmExtendFrameFn = std::mem::transmute(extend_proc);
                    let margins = MARGINS { cxLeftWidth: 1, cxRightWidth: 1, cyTopHeight: 1, cyBottomHeight: 1 };
                    extend_frame(hwnd, &margins);
                }

                let set_attr_proc = GetProcAddress(dwm, b"DwmSetWindowAttribute\0".as_ptr());
                if !set_attr_proc.is_null() {
                    let set_attr: DwmSetAttrFn = std::mem::transmute(set_attr_proc);
                    let policy: u32 = 2; // DWMNCRP_ENABLED
                    set_attr(hwnd, 2, &policy as *const _ as _, 4);

                    let corner: u32 = 2; // DWMWCP_ROUND
                    set_attr(hwnd, 33, &corner as *const _ as _, 4);
                }
            }
        }
    }
}

pub fn window_minimize() {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetActiveWindow() -> *mut std::ffi::c_void;
            fn ShowWindow(hwnd: *mut std::ffi::c_void, cmd: i32) -> i32;
        }
        unsafe {
            let hwnd = GetActiveWindow();
            if !hwnd.is_null() {
                ShowWindow(hwnd, 6);
            }
        }
    }
}

pub fn window_toggle_maximize() {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetActiveWindow() -> *mut std::ffi::c_void;
            fn ShowWindow(hwnd: *mut std::ffi::c_void, cmd: i32) -> i32;
            fn IsZoomed(hwnd: *mut std::ffi::c_void) -> i32;
        }
        unsafe {
            let hwnd = GetActiveWindow();
            if !hwnd.is_null() {
                if IsZoomed(hwnd) != 0 {
                    ShowWindow(hwnd, 9);
                } else {
                    ShowWindow(hwnd, 3);
                }
            }
        }
    }
}

pub fn window_close() {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetActiveWindow() -> *mut std::ffi::c_void;
            fn PostMessageW(hwnd: *mut std::ffi::c_void, msg: u32, wparam: usize, lparam: isize) -> i32;
        }
        unsafe {
            let hwnd = GetActiveWindow();
            if !hwnd.is_null() {
                PostMessageW(hwnd, 0x0112, 0xF060, 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_or_generate_bibtex_from_content() {
        let note_content = r#"---
title: "Test Paper Title"
citekey: "Tester2026"
year: 2026
authors:
  - "Tester, A."
field: "TestField"
---

# Test Paper Title

```bibtex
@article{Tester2026,
  title = {Test Paper Title},
  author = {Tester, A.},
  year = {2026}
}
```
"#;
        let bibtex = extract_or_generate_bibtex(note_content, None).expect("Should extract bibtex");
        assert!(bibtex.contains("@article{Tester2026"));
    }

    #[test]
    fn test_extract_or_generate_bibtex_fallback() {
        let note_content = r#"---
title: "Fallback Title"
citekey: "FallbackKey"
year: 2025
authors:
  - "Author, B."
field: "Quantum"
---

# Notes without codeblock
"#;
        let bibtex = extract_or_generate_bibtex(note_content, None).expect("Should generate bibtex");
        assert!(bibtex.contains("@article{FallbackKey"));
        assert!(bibtex.contains("Fallback Title"));
    }
}
