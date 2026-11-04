# Shared Codebase & Cross-Platform Modifications Reference

This document tracks all modifications made to shared code, cross-platform modules, and UI definitions during the Windows Fluent & Metro UI adaptation (Phase 4). It is written specifically for agents and engineers working on the macOS build to ensure cross-platform parity, prevent regressions, and facilitate seamless debugging.

---

## 1. Overview of Key Changes

| Module / File | Area Changed | Windows Purpose | Potential Impact on macOS | macOS Verification / Parity Notes |
|---|---|---|---|---|
| `src/state.rs` | `save_current_config()` | Added `if cfg!(test) { return; }` to prevent unit tests from polluting user config | None (strictly positive) | Unit tests run in CI/dev on macOS will no longer overwrite `~/Library/Application Support/lx-index/config.yaml`. |
| `src/state.rs` | `remove_vault()` | Case-insensitive & normalized path matching; resets `vault_path = None`, clears library, and resets filters when 0 vaults remain | Behavioral improvement | When deleting the last vault on macOS, the app will cleanly reset to the empty placeholder state rather than leaving stale data in memory. |
| `src/state.rs` | Selection & Filter Toggles | Deselection support for active item, star filters, and perspectives | UI UX enhancement | Deselecting an item now clears `selected_id` and emits clean idle inspector state across platforms. |
| `src/ops.rs` | `choose_vault_folder()` | Added `#[cfg(target_os = "windows")]` branch using `rfd::FileDialog` | Zero impact on macOS | macOS path continues using `osascript -e 'POSIX path of (choose folder ...)'` under `#[cfg(target_os = "macos")]`. |
| `src/ops.rs` | Windows Subclassing | Native Win32 window message handling (`WM_NCCALCSIZE`, `WM_NCHITTEST`, `WM_MOUSEACTIVATE`) for borderless resize and Aero Snap | Zero impact on macOS | Gated strictly behind `#[cfg(target_os = "windows")]`. macOS window decorations / vibrancy remain untouched. |
| `src/main.rs` | Inspector & UI Synchronization | Reset inspector state to empty idle view when `has_selection` is false; terminology updates ("领域", "文献状态") | Parity update | Ensure macOS localization strings match ("Domain"/"Field" and "Literature Status"). |
| `build.rs` | Target OS UI Selection | Compiles `ui/appwindow_windows.slint` on Windows and `ui/appwindow.slint` on macOS/Linux | Decoupled UI architecture | macOS compiles its genuine native HUD UI (`ui/appwindow.slint`) with zero interference from Windows Metro styles. |
| `ui/appwindow.slint` | Restored macOS Design | Restored byte-for-byte to the original macOS HUD layout from `origin/main` | Complete macOS preservation | macOS traffic lights, horizontal HUD status cards, and layout are 100% restored and protected. |
| `ui/appwindow_windows.slint` | Windows 8 Metro UI | Decoupled standalone Windows UI: 3-column continuous layout, bottom app bar, vertical pivot typography, flyout search | Zero impact on macOS | Isolated entirely to Windows builds. |
| `src/main.rs` | Windows UI Property Gating | All Windows-specific UI calls (`set_search_expanded`, `on_window_drag`, `set_window_no_frame`) guarded by `#[cfg(target_os = "windows")]` | Clean macOS compilation | macOS binary builds without referencing Windows-only UI properties. |
| `assets/` | Windows Fluent Icon & App Icon | Added `lx-index-fluent.svg`, `icon_256.png`, and `app_icon.ico` | Zero impact on macOS | macOS `assets/AppIcon.icns` was completely untouched. |

---

## 2. Detailed Technical Breakdown by File

### 2.1 `src/state.rs`

#### A. Guard Test Config Pollution
```rust
pub fn save_current_config(&self) {
    if cfg!(test) {
        return;
    }
    // ...
}
```
- **Reason**: Unit tests (such as `test_app_state_filtering_and_mutation`) invoke `load_vault()` and `rename_vault()`. Previously, this immediately wrote test vault paths and dummy names (e.g., `"My Custom Photonics Vault"`) into the developer's live OS configuration file (`dirs::config_dir()`).
- **macOS Action**: None required. This safeguards macOS developer environments as well.

#### B. 0-Vault Management in `remove_vault()`
```rust
pub fn remove_vault(&mut self, path: &str) -> bool {
    let target_path = PathBuf::from(path);
    let target_canonical = target_path.canonicalize().unwrap_or_else(|_| target_path.clone());
    
    // Matched case-insensitively and canonically
    self.registered_vaults.retain(|v| { ... });

    // When no vaults remain, cleanly reset state:
    if self.registered_vaults.is_empty() {
        self.vault_path = None;
        self.library = LibraryScanResult::default();
        self.active_perspective = "all".to_string();
        self.active_field = None;
        self.active_project = None;
        self.filter_stars.clear();
        self.selected_id = None;
        self.save_current_config();
        return true;
    }
    // ...
}
```
- **Reason**: Previously, removing all managed vault folders left the in-memory library loaded with the previous vault's literature items.
- **macOS Impact**: Positive. Removing all vaults now gracefully returns the app to the empty "Select or add library" placeholder screen on macOS.

#### C. Toggle and Deselection Behavior
- Clicking the active perspective deselects it or returns to `"all"`.
- Clicking the currently selected literature item in the middle list deselects it, transitioning `selected_id` to `None`.
- Star filters can be toggled on and off cleanly.

---

### 2.2 `src/ops.rs`

#### Platform-Segregated File Dialogs
```rust
#[cfg(target_os = "macos")]
pub fn choose_vault_folder() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let script = "POSIX path of (choose folder with prompt \"Select lx-Index Vault Directory\")";
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()?;
    // ...
}

#[cfg(target_os = "windows")]
pub fn choose_vault_folder() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let dialog = rfd::FileDialog::new().set_title("Select lx-Index Vault Directory");
    Ok(dialog.pick_folder())
}
```
- **Reason**: On Windows, executing `powershell` or legacy file dialogs spawned console flashes and suffered from DPI scaling artifacts. Using `rfd` provides a native, modern Windows folder picker.
- **macOS Impact**: None. The macOS `osascript` implementation is preserved verbatim under `#[cfg(target_os = "macos")]`.
- **Note for macOS Agent**: If you wish to migrate macOS away from `osascript` in the future, `rfd::FileDialog` works natively on macOS (via Cocoa `NSOpenPanel`), but the existing `osascript` implementation remains tested and functional.

#### Platform-Segregated Window Framing & Subclassing
- Windows custom frameless styling required Win32 message interception (`WM_NCCALCSIZE`, `WM_NCHITTEST`, `WM_MOUSEACTIVATE`, `WM_SIZE`) in `ops.rs`.
- **Maximized Window Border Elimination**: In `WM_NCCALCSIZE`, instead of delegating to the default window procedure (which produces a 1px white caption border and transparent side gaps on Windows 10/11), the client area is directly mapped to the monitor's work area via `MonitorFromWindow` + `GetMonitorInfoW` (`rgrc[0] = mi.rcWork; return 0;`).
- **Dynamic DWM Frame Toggling**: In `WM_SIZE`, when maximized (`IsZoomed != 0`), DWM margins are set to `{0, 0, 0, 0}` to prevent frame drawing; when restored, margins return to `{1, 1, 1, 1}` to restore subtle drop shadows.
- All Windows subclass functions are enclosed strictly in `#[cfg(target_os = "windows")]`.
- On macOS, standard window controls and `NSWindow` vibrancy styles remain uninhibited.

---

### 2.3 `src/main.rs`

#### A. Idle Inspector View Synchronization
When `selected_id` is `None` (either at startup or when deselecting an item), `sync_ui()` clears all inspector fields:
- `has_selection = false`
- `inspector_id = ""`
- `inspector_title = ""`
- `inspector_authors = ""`
- `inspector_year = 0`
- `inspector_projects = []`
- `inspector_tags = []`
- `inspector_has_pdf = false`
- `inspector_has_note = false`

#### B. Placeholder String Refinement
When no vault is open:
- Chinese: `"在左下角选择或添加文献库"`
- English: `"Select or add a library in bottom-left"`
Clarifies that the vault cannot be clicked directly in the top header and instructs users where to add one.

#### C. Terminology Refinement
- "学科" -> "领域" ("Domain" / "Field")
- "文献视角" -> "文献状态" ("Literature Status")
Please verify that macOS UI labels conform to these terms.

---

### 2.4 UI Architecture Decoupling: macOS vs Windows

The UI architecture has been cleanly decoupled to ensure that Windows 8 Metro design enhancements do not interfere with macOS designs:

1. **macOS UI (`ui/appwindow.slint` and `ui/appwindow_macos.slint`)**:
   - Restored directly from `origin/main` without modification.
   - Preserves Apple HIG standards, horizontal HUD cards, top status header, and unified macOS traffic lights.

2. **Windows Metro UI (`ui/appwindow_windows.slint`)**:
   - Continuous 3-column Metro layout without horizontal dividers.
   - Left column Metro Pivot layout (vertical Panorama/Pivot typography with bold active state).
   - Middle column: Metro bottom App Bar with circular icons, label subtitles, flyout search with animated expansion, and auto-dismiss click backdrop.
   - Settings modal with Panorama sliding animations, Windows 11 Fluent 3D logo, and version display (`v0.6.0`).

3. **Build-Time Dispatch (`build.rs`)**:
   ```rust
   #[cfg(target_os = "windows")]
   let ui_file = "ui/appwindow_windows.slint";

   #[cfg(not(target_os = "windows"))]
   let ui_file = "ui/appwindow.slint";

   slint_build::compile(ui_file).expect("Slint build failed");
   ```
   Compiles the corresponding native UI definition per target OS, avoiding any cross-platform styling regressions.

---

## 3. macOS Parity Checklist & Regression Tests

When building or reviewing the macOS target, check the following:

1. **Compilation**:
   ```bash
   cargo check
   cargo test
   ```
   Ensure all 9 unit tests pass and `cargo test` does NOT overwrite `~/Library/Application Support/lx-index/config.yaml`.

2. **Empty Startup Experience**:
   - Delete `~/Library/Application Support/lx-index/config.yaml` or run with an empty configuration.
   - Verify that the app launches with 0 vaults, shows `"Select or add a library in bottom-left"`, and displays an empty document table.

3. **Vault Deletion**:
   - Add a test vault, then remove it in Settings -> Vaults.
   - Verify that upon removing all vaults, the app returns to the clean 0-vault placeholder without retaining cached items in memory.

4. **Folder Picker**:
   - Click "Add Vault" in the bottom-left App Bar.
   - Verify that macOS displays the native Cocoa folder picker sheet without errors.

5. **List Deselection**:
   - Click a literature row in the middle list (inspector populates).
   - Click the same row again: verify the selection clears and the right inspector returns to its idle placeholder state.

---

*Document version: 0.6.0 — Created for cross-platform agent coordination.*
