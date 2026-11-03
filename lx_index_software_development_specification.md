# lx Index - System Architecture & Development Specification (Rust + Slint)

**Product Name**: `lx Index`  
**Brand Identity**: Lucentia (`lx`) Aesthetics Laboratory  
**Architecture Type**: Pure Compiled Native Desktop Application (Zero WebView, Zero Browser Runtime)  
**Core Technologies**: **Rust 1.80+** & **Slint UI Toolkit** (Skia / Metal / Direct3D hardware-accelerated backends)  
**Target Operating Systems**:

- **macOS Primary (Tier 1)**: macOS 15.8 (Apple Silicon M-Series / M4 Architecture), Apple Human Interface Guidelines (HIG).
- **Windows Target (Tier 2)**: Windows 10 (Version 22H2 / Build 19041+), Metro / Modern UI & Fluent Design aesthetic.

---

## 1. Architectural Philosophy & Engine Selection

### 1.1 The "Pure Native" Mandate

1. **Zero Browser Dependency**: No WebKit, no Chromium, no Electron, and no Edge WebView2. The entire user interface is rendered directly via GPU graphics pipelines (Metal on macOS, Direct3D/Vulkan on Windows) driven by compiled Rust machine code.
2. **Minimalist Runtime Profile**:
   - Idle RAM: `< 20 MB`.
   - Cold Startup Time: `< 150 ms`.
   - Standalone Binary: Single self-contained executable (~8–15 MB), zero external runtime dependencies.
3. **Filesystem as the Sole Database**:
   - No SQLite, DuckDB, or indexed database caches on disk.
   - The directory hierarchy (`<Field>/`) and the YAML frontmatter in companion Markdown notes (`notes/*.md`) constitute the absolute Single Source of Truth (SSOT).
4. **Viewer-Agnostic File Management**:
   - No built-in PDF reader or full-featured Markdown editor.
   - Primary interactions are OS-native reveals:
     * macOS: `open -R <path>` (Reveal in Finder).
     * Windows: `explorer.exe /select,"<path>"` (Select in File Explorer).
5. **Non-Destructive Frontmatter Mutation**:
   - Updating status, ratings, tags, or projects strictly mutates the YAML frontmatter block without altering or damaging human annotations (`## 0. Personal Insights`) or AI summaries.

---

## 2. Technology Stack & Rust Crates

### 2.1 Core Dependencies (`Cargo.toml`)

```toml
[package]
name = "lx-index"
version = "0.1.0"
edition = "2021"

[dependencies]
# GUI Framework
slint = { version = "1.8", features = ["renderer-skia", "compat-1-2"] }

# Filesystem Crawling & Watching
walkdir = "2.5"
notify = "6.1"

# Serialization & Markdown Frontmatter
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"

# System Integration
open = "5.3"           # Cross-platform reveal / open
trash = "5.1"          # Native OS Trash / Recycle Bin
arboard = "3.4"        # Native system clipboard (BibTeX & Citekey copy)
dirs = "5.0"           # Standard system directories config

[build-dependencies]
slint-build = "1.8"
```

---

## 3. Physical File System Contract

### 3.1 Vault Topography

The application makes on an arbitrary directory designated by the user (typically synchronized via OneDrive: `'OneDrive-PolitecnicodiMilano/Polimi PhD/03_Resources/Literature'`), application itself operate at a independ folder path, not in the vault path. Like Obsdian, use can set a specific directory as a vault, (and can switch in the application), here is an example structure for an user vault:

```text
<Library_Root>/
├── Optical_Analog_Computing/             <-- Field Folder (Research Domain)
│   ├── notes/
│   │   ├── 2026_Shen_OpticalNeuralNet.md
│   │   └── 2025_Bandyopadhyay_Hardware.md
│   ├── 2026_Shen_OpticalNeuralNet.pdf
│   └── 2025_Bandyopadhyay_Hardware.pdf
│
├── Quantum_Optics/                       <-- Field Folder (Research Domain)
│   ├── notes/
│   │   └── 2026_Bollani_GiantThermoOptic.md
│   └── 2026_Bollani_GiantThermoOptic.pdf
│
└── .lxindex/                             <-- Local Vault Config
    └── config.json                       <-- Active library paths & UI state
```

### 3.2 Naming Protocol

* **PDF File**: `[YYYY]_[FirstAuthor]_[TitleKeyword].pdf`
  - *Example*: `2026_Bollani_GiantThermoOptic.pdf`
  - *Constraint*: Starts with a 4-digit year. The `TitleKeyword` is concise (2–4 words) to prevent Windows 10 `MAX_PATH` (260-character) truncation errors.
* **Companion Markdown Note**: Stored in the immediate `notes/` subdirectory, sharing the exact same basename:
  - *Example*: `notes/2026_Bollani_GiantThermoOptic.md`

---

## 4. Data Model & State Machine

### 4.1 In-Memory Rust Data Structures

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PaperMetadata {
    pub title: String,
    pub citekey: Option<String>,
    pub doi: Option<String>,
    pub year: u32,
    pub authors: Vec<String>,
    pub field: String,
    pub projects: Vec<String>,
    pub status: PaperStatus,
    pub importance: u8, // 1 to 5 stars
    pub tags: Vec<String>,
    pub pdf_path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PaperStatus {
    Unread,     // Newly imported, companion stub note created
    AiSkimmed,  // Processed by literature reading agent
    DeepRead,   // Digested by researcher with personal comments
    Discarded,  // Low relevance, hidden from primary views
}
```

### 4.2 Slint Shared Model (`ui/models.slint`)

```slint
export struct SlintPaperItem {
    id: string,
    filename: string,
    title: string,
    authors: string,
    year: int,
    field: string,
    projects_display: string,
    tags_display: string,
    status_code: int, // 0: Unread, 1: AiSkimmed, 2: DeepRead, 3: Discarded
    importance: int,  // 0 to 5
    has_pdf: bool,
    has_note: bool,
    is_missing_pdf: bool,
}
```

---

## 5. Slint UI Architecture & Adaptive Styling

The UI is defined using Slint markup files (`.slint`) and compiled directly into native C++/Rust binary code at build time.

### 5.1 Three-Pane Desktop Layout

```text
+-----------------------+------------------------------------------+------------------------------+
| SIDEBAR               | MAIN LIST VIEW                           | METADATA INSPECTOR           |
+-----------------------+------------------------------------------+------------------------------+
| [Library Selector]    | Search: [Q Filter title, tags, authors]  | Full Article Title           |
|                       +------------------------------------------+ Authors / Year / Journal     |
| PERSPECTIVES          | [★ 4] 2026_Bollani_GiantThermoOptic      | ---------------------------- |
| - All Items           |       Giant thermo-optic tuning in...    | Status: [ ai-skimmed    V ]  |
| - Inbox (Unread)      |       Field: Quantum_Optics              | Rating: [ ★ ★ ★ ★ ☆ ]        |
| - High Priority (★4+) |       Projects: SoftMAX | Tags: a-Si...  | Field:  [ Quantum_Optics V ] |
| - Discarded           |                                          | Projects:                    |
|                       | [★ 5] 2026_Shen_OpticalNeuralNet         |   [+ Add]  SoftMAX (x)       |
| FIELDS (Directories)  |       Deep learning with coherent...     | Tags:                        |
| - Optical_Analog      |       Field: Optical_Analog_Computing    |   [+ Add]  metasurface (x)   |
| - Quantum_Optics      |                                          | ---------------------------- |
|                       |                                          | [ Reveal PDF in OS Folder  ] |
| PROJECTS (Frontmatter)|                                          | [ Reveal Note in OS Folder ] |
| - 01_SoftMAX          |                                          | [ Copy BibTeX Citation     ] |
| - 02_Flatband         |                                          | [ Copy Citekey (@Bollani)  ] |
+-----------------------+------------------------------------------+------------------------------+
```

### 5.2 Adaptive Visual Themes (macOS 15.8 vs. Windows 10)

Slint allows dynamic styling tokens configured via Rust at initialization:

* **macOS 15.8 Profile**:
  - **Typography**: `-apple-system`, `SF Pro Display`, `SF Pro Text`.
  - **Geometry**: Continuous rounded corners (`border-radius: 8px` to `12px`), standard macOS list selection highlights.
  - **Color Palette**: Dark translucent slate, subtle hairline borders (`#FFFFFF1A`).
* **Windows 10 Metro / Modern UI Profile**:
  - **Typography**: `Segoe UI Variable`, `Segoe UI`.
  - **Geometry**: Sharp or micro-rounded corners (`border-radius: 2px` to `4px`), geometric tile cards.
  - **Color Palette**: High-contrast dark acrylic simulation, accent borders with theme color (e.g., `#0078D7` or custom Lucentia gold `#C5A059`).

---

## 6. Rust Backend Module Specifications

### 6.1 Directory Scanner & Pairing (`src/scanner.rs`)

1. **Field Discovery**: Scan all 1st-level directories within the designated library root. Directory names represent physical **Fields**.
2. **File Pairing Algorithm**:
   - Collect all `.pdf` files located directly inside `<Field>/`.
   - Check if `<Field>/notes/<basename>.md` exists:
     * If note exists: Parse YAML frontmatter using `frontmatter.rs`.
     * If note does NOT exist: Queue as `Pending Ingestion / Unread`.
   - Check for orphan notes (notes existing without a matching `.pdf`): Flag as `is_missing_pdf = true`.
3. **Safe Deletion Protocol**:
   - Calling delete moves both `<basename>.pdf` and `notes/<basename>.md` to the OS Trash using `trash::delete_file()`.

### 6.2 Frontmatter Parser & Mutator (`src/frontmatter.rs`)

Must guarantee **lossless round-trip modifications**:

1. Read file content and extract YAML delimiters `--- ... ---`.
2. Deserialize into `PaperMetadata`.
3. When saving modifications (e.g., changing status or importance):
   - Mutate only the YAML block.
   - Retain the exact remainder of the file intact (preserving `# Title`, `## 0. Personal Insights`, summaries, and BibTeX codeblocks).

### 6.3 System OS Operations (`src/ops.rs`)

```rust
pub fn reveal_in_file_manager(path: &std::path::Path) -> std::io::Result<()> {
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
    Ok(())
}

pub fn copy_bibtex_to_clipboard(note_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(note_path)?;
    // Extract ```bibtex ... ``` block or generate on-the-fly from frontmatter
    let bibtex = extract_or_generate_bibtex(&content)?;
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(bibtex)?;
    Ok(())
}
```

---

## 7. Implementation Roadmap for Coding Agents

### Phase 1: macOS First-Pass Scaffolding (macOS 15.8)

- [x] Initialize Cargo project with Slint dependencies (`slint = "1.8"`).
- [x] Implement `src/scanner.rs`: Read directory trees, pair PDFs with notes.
- [x] Implement `src/frontmatter.rs`: Lossless extraction and updating of YAML headers.
- [x] Implement `src/ops.rs`: Finder reveal commands (`open -R`), system clipboard actions, and safe file deletion (`trash`).

### Phase 2: Slint UI Layout & Reactive Binding

- [x] Design `ui/appwindow.slint`: 3-pane responsive desktop shell.
- [x] Create `SlintPaperItem` struct and bind `slint::VecModel` to the center list view.
- [x] Implement reactive search filtering (matching query against title, authors, tags, and citekey).
- [x] Implement right-hand Inspector pane: Two-way binding for `status`, `importance` stars, and projects list.

### Phase 3: Ingestion & Stub Creation Flow

- [x] Detect new PDFs that lack companion notes.
- [x] Implement "Import All / Import Selected" button:
  * Generates `notes/<basename>.md` with initial YAML frontmatter (`status: unread`).
  * Emits refresh event to reload the list view instantly.

### Phase 4: Windows 10/11 Fluent & Metro Adaptation

- [ ] Verify clean compilation on Windows (`cargo build --target x86_64-pc-windows-msvc`).
- [ ] Adapt UI styling parameters for Windows: Apply `Segoe UI Variable`, crisp micro-rounded corners (4px), and Metro / Fluent cards.
- [ ] Test Explorer reveal command (`explorer /select,`) and normalize path separators (`\` vs `/`).
- [ ] Embed `assets/app_icon.ico` into `lx-index.exe` via `winres` in `build.rs`.

### Phase 5: Production Packaging

- [x] Package macOS universal binary and bundle into `.app` / `.dmg` (Completed in v0.5.0).
- [ ] Package Windows standalone portable executable (`lx-index.exe`) and installer/zip.

---

## 8. Windows Implementation Specification & Functional Parity Contract (Phase 4 Agent Guide)

> [!IMPORTANT]
> **For the Windows Development Agent**:
> The macOS primary implementation has reached milestone **v0.5.0 (Beta)**. The entire Rust backend (`src/scanner.rs`, `src/frontmatter.rs`, `src/state.rs`, `src/watcher.rs`, `src/ops.rs`, `src/i18n.rs`) is production-tested, fully unit-tested, and platform-agnostic.
> When developing the Windows version, **you must maintain 100% data and functional parity** with the backend contract outlined below. While the UI should adopt native Windows Fluent/Metro aesthetics, the underlying library structures, YAML frontmatter schemas, and business workflows must remain identical so that researchers synchronizing vaults via OneDrive between macOS and Windows experience zero friction.

### 8.1 Core Functional Contracts (Strict Parity Required)

#### 1. Vault Management & Multi-Vault Registry
* **Configuration Path**: Persisted via `dirs::config_dir()`.
  - On Windows, automatically resolves to: `%APPDATA%\lx-index\config.yaml` (e.g. `C:\Users\<User>\AppData\Roaming\lx-index\config.yaml`).
  - Serialized via `serde_yaml` using the `AppConfig` struct:
    ```rust
    pub struct AppConfig {
        pub last_vault_path: Option<String>,
        pub vaults: Vec<VaultRecord>,
        pub theme_id: Option<String>,
        pub language: Option<String>,
    }
    ```
* **Custom Vault Renaming (Obsidian Pain Point Solution)**:
  - Each vault record holds `pub name: String` (display alias) and `pub path: String` (physical disk path).
  - Users can arbitrarily rename the vault display alias in the Vaults Manager modal without renaming or moving the folder on disk.
  - The Windows UI must display `vault.name` (falling back to folder name if unset) in the top header and vaults manager.
* **Unmanaging vs. Deleting**:
  - "Unmanage Vault" (`remove_vault`) strictly removes the vault from `registered_vaults` and configuration. It **NEVER** deletes files from the user's hard drive.

#### 2. Lossless YAML Frontmatter Protocol
* **Frontmatter Schema**:
  ```yaml
  ---
  title: "Coherent Optical Computing"
  citekey: "Author2026Coherent"
  doi: "10.1117/12.345678"
  year: 2026
  authors:
    - "Author, A."
    - "Researcher, B."
  field: "Optical_Computing"
  projects:
    - "SoftMAX"
  status: unread       # unread | ai-skimmed | deep-read | discarded
  importance: 4        # 0 to 5
  tags:
    - "optics"
    - "neural-network"
  summary: "Key insight or personal note takeaway"
  ---
  ```
* **Lossless Roundtrip Guarantee**:
  - `src/frontmatter.rs` must only replace the string between the opening and closing `---` delimiters.
  - Human annotations (`## 0. Personal Insights`), AI notes, reading logs, and embedded ````bibtex ... ```` code blocks in the Markdown body must be preserved down to the exact byte.

#### 3. Automatic Discard & Priority State Machine
* **Auto-Discard Workflow**: When the user sets `status` to `Discarded`, the importance rating is automatically clamped/tagged as **2 Stars (★2 = Discarded)**.
* **Deletion Workflow (★1)**: When an item is marked **1 Star (★1 = To Delete)**, it represents literature flagged for trash. Triggering deletion calls `safe_delete_paper()`, which sends both the `.pdf` and companion `notes/<basename>.md` to the Windows Recycle Bin via the `trash` crate.
* **High Priority Filter (★4+)**: Items with importance `>= 4` (★4 Important, ★5 High Priority) automatically qualify for the `High Priority` perspective in the sidebar.
* **Summary / Appendix Field**: The `summary` YAML field represents personal takeaways and quick reference notes. It is displayed as an italicized block in the middle column list and is directly editable in the Inspector pane with instant lossless persistence.

#### 4. Realtime Filesystem Watching
* `notify::RecommendedWatcher` monitors the active vault root recursively.
* Emits debounced reload events (500ms debounce) when files are created, updated, or removed externally (e.g. by Obsidian, Zotero, or OneDrive sync).
* On Windows, handle `ReadDirectoryChangesW` buffer overflow gracefully by triggering a full rescan when a watcher error occurs.

#### 5. Ingestion & Stub Creation Flow
* When new `.pdf` files are deposited in any `<Field>/` directory without companion notes:
  - System discovers them with `has_pdf: true, has_note: false`.
  - Displayed in the `Pending Ingestion` perspective and top banner (`uncatalogued PDF(s) detected`).
  - "Import Note" / "Import All" generates `notes/<basename>.md` containing:
    ```markdown
    ---
    title: "<Parsed Title>"
    citekey: "<AuthorYearKeyword>"
    year: <Year>
    authors:
      - "<Author>"
    field: "<Field>"
    projects: []
    status: unread
    importance: 0
    tags: []
    ---

    # <Title>

    ## 0. Personal Insights
    ```

#### 6. Multi-Select Star Rating Filter
* The center list view supports multi-select star filtering (TreeSelect style), enabling simultaneous selection of arbitrary star combinations (e.g. `[5, 4]` for high priority or `[1]` for cleanup).

#### 7. System Operations & Clipboard
* **Explorer Reveal**:
  ```rust
  #[cfg(target_os = "windows")]
  std::process::Command::new("explorer")
      .arg(format!("/select,\"{}\"", path.display()))
      .spawn()?;
  ```
* **BibTeX Copying**: Extracts embedded ````bibtex ... ```` block from note or generates fallback `@article{...}` based on YAML frontmatter.
* **Citekey Copying**: Copies `@<citekey>` string directly to the Windows clipboard via `arboard`.
* **Recycle Bin**: Uses `trash::delete()` which invokes the native Windows COM `IFileOperation` interface, ensuring items appear in the Windows desktop Recycle Bin.

---

### 8.2 Windows-Specific Technical Requirements

1. **Path Normalization & Drive Letters**:
   - Always handle Windows backslashes (`\`) vs UNIX slashes (`/`).
   - Store canonical paths or handle drive letter case normalization (`C:\` vs `c:\`).
   - Support long paths (`\\?\` UNC prefix) for nested literature hierarchies in OneDrive PhD repositories.
2. **Windows Application Icon Embedding**:
   - The Windows icon has already been created and verified at [`assets/app_icon.ico`](file:///Users/dai/Documents/VibeCoding/lx-Index/assets/app_icon.ico) (containing 256×256, 128×128, 64×64, 48×48, 32×32, 24×24, 16×16 with full 32-bit alpha transparency).
   - In `build.rs` on Windows, use the `winres` crate to compile `assets/app_icon.ico` directly into the `.exe` binary resource table:
     ```rust
     #[cfg(target_os = "windows")]
     {
         let mut res = winres::WindowsResource::new();
         res.set_icon("assets/app_icon.ico");
         res.set("ProductName", "lx Index");
         res.set("FileDescription", "Literature Management HUD");
         res.set("LegalCopyright", "© 2026 Lucentia");
         res.compile().unwrap();
     }
     ```
3. **Slint Windows GPU Acceleration**:
   - Features in `Cargo.toml`: `slint = { version = "1.8", features = ["renderer-skia"] }`.
   - On Windows, Skia automatically selects Direct3D 12/Direct3D 11 or Vulkan hardware acceleration with software fallback.
4. **Windows Native Fluent / Metro UI Design Recommendations**:
   - **Corners**: Micro-rounded corners (`border-radius: 4px`) conforming to Windows 11 Fluent Design or sharp rectangular tiles (`border-radius: 0px` to `2px`) for Windows 10 Metro.
   - **Typography**:
     - UI Font: `Segoe UI Variable Text`, `Segoe UI`, `Microsoft YaHei` (微软雅黑).
     - Brand Logo Serif: `Georgia`, `Cambria`, or bundled Baskerville.
   - **Window Framing & DWM**:
     - Maintain compatibility with Windows 11 Snap Layouts (hovering over maximize button).
     - Top titlebar integration can utilize custom DWM styling or Windows native caption controls.
5. **Localization & Color Themes**:
   - Maintain full dual-language support (`zh` / `en`) via `src/i18n.rs`.
   - Support the 5 scholarly literary themes:
     1. `羊皮古雅 · Parchment Warm (Default)`
     2. `暮光晨曦 · Rosé Pine Dawn`
     3. `经典学术 · Solarized Light`
     4. `象牙古籍 · Antique Ivory`
     5. `清雅现代 · Catppuccin Latte`
6. **About Modal & Brand Information**:
   - Version: `0.5.0`
   - Brand & Studio: `Designed & Developed by Lucentia`
   - Tagline: `Literature HUD & Companion Workspace` (文献管理 HUD 与伴生工作空间)
   - Copyright: `© 2026 Lucentia. All rights reserved.`