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

## 4. Phase 5: macOS True 3-Column UI & Native System Integration (v0.6.5)

This section documents the macOS platform-specific UI refinements and shared codebase coordination introduced in v0.6.5.

### 4.1 Cross-Platform Version Synchronization
- **Issue**: Previously, version numbers were hardcoded independently in `ui/appwindow_macos.slint` (`"v0.5.0 Beta"`) and `ui/appwindow_windows.slint` (`"v0.6.0"`), causing settings modal version desynchronization when bumping versions in `Cargo.toml`.
- **Solution**:
  - Declared `in-out property <string> app-version: "v0.6.5";` in `AppWindow` across all Slint definitions (`appwindow_macos.slint`, `appwindow.slint`, `appwindow_windows.slint`).
  - In `sync_ui()` (`src/main.rs`), dynamically bind `ui.set_app_version(format!("v{}", env!("CARGO_PKG_VERSION")).into());`.
  - Settings About tabs on both macOS and Windows now reference `root.app-version` dynamically. Any version update in `Cargo.toml` is automatically propagated to both platforms without manual edits.

### 4.2 UI Architecture Decoupling (`build.rs`)
- Explicit target dispatch configured in `build.rs`:
  ```rust
  #[cfg(target_os = "windows")]
  let ui_file = "ui/appwindow_windows.slint";

  #[cfg(target_os = "macos")]
  let ui_file = "ui/appwindow_macos.slint";

  #[cfg(not(any(target_os = "windows", target_os = "macos")))]
  let ui_file = "ui/appwindow.slint";
  ```
- Isolates macOS-specific UI definitions (`ui/appwindow_macos.slint`) from Windows Metro styles (`ui/appwindow_windows.slint`) and fallback desktop builds (`ui/appwindow.slint`), ensuring clean separation of design languages.

### 4.3 macOS True 3-Column Continuous Layout
- **Top Bar Elimination**: Removed the overarching horizontal top header with distinct crust background (`root.theme.crust`) and top horizontal divider. All 3 columns now run continuously from top to bottom:
  - **Left Sidebar** (`root.theme.mantle`):
    - Top: macOS traffic lights clearance (`padding-left: 92px`) + literary brand logo (`lx Index`).
    - Middle: Scrollable navigation tree for perspectives, domains, and projects.
    - Seamless Layout: Removed all internal horizontal dividers (between brand header, navigation list, and bottom controls), achieving a completely unified, seamless column.
    - Bottom: App control bar containing Vault selector button & Reload button on the bottom-left corner, and Settings gear button on the bottom-right corner (adjacent to the divider with the middle column). Button designs remain identical to existing specifications.
  - **Middle Column** (`root.theme.base`):
    - Top: Unified 52px search HUD bar with search input, star filter dropdown, and reset button.
    - Middle: Literature card list.
    - Bottom: Document count status bar (`root.status-bar-text`, e.g., `"显示 12 篇文献 • 共 12 篇"`).
  - **Right Column** (`root.theme.mantle`):
    - Continuous inspector pane with metadata, actions, and note stubs.

### 4.4 Search Input Focus & Blur Restoration
- **Problem**: When the search input in the middle column lost user focus, it remained in the active state with a mauve highlight border and blinking cursor due to Slint's default focus retention on non-focusable click targets.
- **Solution**:
  - Introduced `dummy-focus := FocusScope {}` in `AppWindow`.
  - Bound `dummy-focus.focus()` to search input Enter (`accepted`), clear button (`xmark`), list card selection (`card-touch`), sidebar category items, star filter popover, and middle pane background clicks.
  - Calling `dummy-focus.focus()` immediately resets `search-input.has-focus` to `false`, restoring the search box to its dormant border and appearance.

### 4.5 Native macOS System Menu Bar & Comprehensive Window Control Operations
- **Problem**: macOS running headless binaries or Slint windows lacks system-level window control menus in the system menu bar, missing native macOS window management (Fill, Center, Tiling / Half & Quarter split screens). Furthermore, menu items could become disabled or disappear when windows were minimized and reopened due to AppKit `setWindowsMenu` lifecycle resets and default menu auto-enable validation.
- **Solution**:
  - Implemented `setup_macos_menu_bar(lang: &str)` in `src/vibrancy.rs` using Cocoa / Objective-C runtime:
    - **App Menu** (`lx-Index`): About, Hide (`Cmd+H`), Hide Others (`Opt+Cmd+H`), Show All, Quit (`Cmd+Q`).
    - **Edit Menu** (`编辑` / `Edit`): Undo (`Cmd+Z`), Redo (`Cmd+Shift+Z`), Cut (`Cmd+X`), Copy (`Cmd+C`), Paste (`Cmd+V`), Select All (`Cmd+A`).
    - **Window Menu** (`窗口` / `Window`):
      - 最小化 / Minimize (`performMiniaturize:`, `Cmd+M`)
      - 缩放 / Zoom (`performZoom:`)
      - 填充屏幕 / Fill (`lxFill:` - smooth animation to active screen's `visibleFrame`)
      - 居中窗口 / Center (`lxCenter:` - repositions active window to screen center)
      - 移动与调整大小 / Move & Resize (Submenu):
        - 平铺到屏幕左侧 / Left (`lxLeft:`)
        - 平铺到屏幕右侧 / Right (`lxRight:`)
        - 平铺到屏幕上方 / Top (`lxTop:`)
        - 平铺到屏幕下方 / Bottom (`lxBottom:`)
        - 左上分屏 / Top Left (`lxTopLeft:`)
        - 右上分屏 / Top Right (`lxTopRight:`)
        - 左下分屏 / Bottom Left (`lxBottomLeft:`)
        - 右下分屏 / Bottom Right (`lxBottomRight:`)
      - 进入全屏幕 / Toggle Full Screen (`toggleFullScreen:`, `Ctrl+Cmd+F`)
      - 关闭窗口 / Close Window (`performClose:`, `Cmd+W`)
      - 前置全部窗口 / Bring All to Front (`arrangeInFront:`)
  - Registered dynamic Objective-C class `LxWindowActionHandler` to dispatch window sizing and positioning math with Cocoa `visibleFrame` and animated frame transitions.
  - Implemented `validateMenuItem:` returning `true` and disabled `autoenablesItems` to ensure all menu items remain permanently active and never get grayed out when minimizing or changing focus.
  - Automatically de-miniaturizes the window (`deminiaturize:`) if any window command is executed while minimized.
  - Updated `scripts/package_macos.sh` to configure `CFBundleDevelopmentRegion = zh_CN` and `CFBundleLocalizations = [zh_CN, en]` in `Info.plist`.
  - Fully internationalized (authentic Apple native Chinese & English) and updated dynamically on language switch in settings.

### 4.6 Bug Fix: Projects & Tags Parameter Addition in Inspector
- **Problem**: The project and tag input boxes in the right inspector were unclickable and could not receive text input.
- **Root Cause**: `TextInput` was placed inside an unconstrained `HorizontalLayout` with `alignment: center`. With an empty initial text (`""`), the layout engine collapsed the `TextInput` preferred width to 0px, causing all mouse clicks to hit the unhandled parent `Rectangle`.
- **Solution**:
  - Removed `alignment: center` and sized `TextInput` explicitly to `x: 8px; y: 0; width: parent.width - 16px; height: parent.height;` inside a `horizontal-stretch: 1;` container.
  - Added active focus border highlighting (`mauve` on macOS, `accent` on Windows).
  - Allowed both Enter key and `+ Add` button to commit projects and tags directly into paper frontmatter.

### 4.7 UI Refinements & Micro-Interactions (Phase 5 Polish)
- **Search Box Full-Area Activation**: Added a transparent `TouchArea` to the search input container and gave `TextInput` `horizontal-stretch: 1;`, allowing clicks anywhere inside the search box bounds to focus the input.
- **Font Descender Clipping Fix**: Prevented CJK/Latin font descender cutoff (letters `g`, `y`, `p`) in the Theme settings tab by assigning explicit `height: 20px; vertical-alignment: center;` to theme title elements and expanding card height to `58px`.
- **Tag vs. Project Color Differentiation**: Differentiated projects (`teal`, 汉玉青绿 with target icon) from tags (`peach`, 陶土赭石 with `#` prefix) across both the middle document list and the right inspector.
- **Inspector Tag Cloud Wrapping (`FlexboxLayout`)**: Converted single-column vertical tag lists into dynamic horizontal wrapping tag clouds (`FlexboxLayout { flex-wrap: wrap; spacing: 6px; }`).
- **TagChip Delete Cross Alignment**: Replaced text glyphs (`"×"`) with symmetric vector icons (`icons/xmark.svg`) and locked outer circular badges to vertical center (`y: (parent.height - self.height) / 2;`), achieving pixel-perfect alignment with tag text.
- **Uncatalogued PDF Prompt Banner Overflow Fix**: Restructured the uncatalogued document banner from a single unconstrained horizontal row to a 2-row `VerticalLayout` (Row 1: Title and `HudButton` aligned with `space-between`; Row 2: description text with `wrap: word-wrap;`). Removed the hardcoded `height: 60px;` to let the container dynamically size according to text wrapping, completely preventing the "新建伴生笔记" button from overflowing the inspector pane boundary.
- **Middle Column Ingestion Banner Relocation**: Moved the uncatalogued PDF ingestion banner in the middle pane from the top of the column to directly below the Search HUD Bar and star filter row (separated by a 1px border divider), aligning with the Windows layout hierarchy and keeping the search & filter bar anchored at the top of the middle column. Corrected the star rating dropdown popover `y` offset to a fixed `50px`.
- **Concentric Tall Titlebar & Native Apple Fullscreen Architecture**:
  - **Windowed Mode**: Configured `NSWindowToolbarStyleUnified (3)` and attached a transparent `NSToolbar` (with `showsBaselineSeparator = false`), natively placing the traffic lights at (19.5pt, 18~20pt) from top-left, forming an exact concentric circle with macOS window's 18~20pt corner radius (matching Safari, Finder, and System Settings). Left brand logo maintains a clean, static `padding-left: 92px` reserve spacing. Window title is hidden (`NSWindowTitleHidden`) so custom Slint branding is unobstructed.
  - **Compact Fullscreen Titlebar (`NSWindowToolbarStyleUnifiedCompact`)**: When entering fullscreen, dynamically toggles `toolbarStyle` to `NSWindowToolbarStyleUnifiedCompact (4)`, `titleVisibility: NSWindowTitleVisible (0)`, and `titlebarAppearsTransparent: false`. macOS natively renders a sleek, compact (~28–38pt) drop-down titlebar with centered app title `"lx Index"` on native frosted material (matching Apple native apps like Notes and Pages) rather than an oversized 52pt header.
  - **Zero-Jump Direct Fullscreen Exit**: Because the toolbar geometry was never artificially collapsed or detached, AppKit's CoreAnimation transition engine animates directly from the fullscreen state back to the windowed state without any secondary resizing pass. The traffic lights appear directly in their concentric 52pt tall titlebar position without any jump or flash. Both `NSWindowWillExitFullScreenNotification` and `NSWindowDidExitFullScreenNotification` restore `titleVisibility = NSWindowTitleHidden`, `titlebarAppearsTransparent = true`, and re-activate `NSWindowToolbarStyleUnified (3)`.
  - **Presentation Options**: Window delegate injects `window:willUseFullScreenPresentationOptions:` returning `NSApplicationPresentationAutoHideToolbar | NSApplicationPresentationAutoHideMenuBar | NSApplicationPresentationFullScreen`, ensuring clean autohiding of system chrome.
- **Native Inactive Grey Traffic Lights (Layer-Backed Opaque Overlays)**:
  - Configured `[window setBackgroundColor:[NSColor windowBackgroundColor]]` instead of `clearColor`.
  - Injected native 12x12pt `CALayer` overlays on each standard window button (`_NSThemeWidget` for close, miniaturize, and zoom) with macOS-spec solid disabled grey fill (`#CECECE` / `NSColor(white: 0.81, alpha: 1.0)`) and subtle border (`#ADADAD` / `NSColor(white: 0.68, alpha: 1.0)`).
  - Listens to `NSWindowDidResignKeyNotification`, `NSWindowDidBecomeKeyNotification`, `NSApplicationDidResignActiveNotification`, and `NSApplicationDidBecomeActiveNotification`.
  - When the window or application loses focus (inactive), the overlays are immediately unhidden (`isHidden = false`), reliably displaying crisp native macOS disabled grey circles even when DarkAqua mode draws 14% translucent white on light parchment backgrounds. When the window regains key focus (active), the overlays are immediately hidden (`isHidden = true`), revealing the standard Apple red, yellow, and green traffic lights with zero latency. Button hit testing and window actions remain 100% native.
- **Menu Bar Persistence Across Spaces & Minimizing**:
  - Registered `NSApplicationDidBecomeActiveNotification`, `NSWindowDidBecomeKeyNotification`, and `NSWindowDidDeminiaturizeNotification` to re-assert `[NSApp setMainMenu:GLOBAL_MAIN_MENU]` whenever the app or window regains focus across virtual desktops (Spaces) or after minimizing.
  - Explicitly configured `NSApplicationActivationPolicyRegular (0)` and preserved `NSWindowCollectionBehaviorParticipatesInCycle (32)` to ensure macOS WindowServer retains the app within Space switching cycles.
- **Native Titlebar Double-Click (Minimize / Zoom)**:
  - Swizzled `sendEvent:` on the window class to intercept double-click events in the top 52px Tall Titlebar header.
  - Dynamically reads the user's macOS system preference `AppleActionOnDoubleClick` from `NSUserDefaults` (*System Settings > Desktop & Dock > Double-click a window's title bar to...*).
  - Automatically invokes `performMiniaturize:` (if set to Minimize or default) or `performZoom:` (if set to Zoom/Maximize).

---

*Document version: 0.6.5 — Updated for macOS Phase 5 & cross-platform synchronization.*
