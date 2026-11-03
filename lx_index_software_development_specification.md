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

- [ ] Initialize Cargo project with Slint dependencies (`slint = "1.8"`).
- [ ] Implement `src/scanner.rs`: Read directory trees, pair PDFs with notes.
- [ ] Implement `src/frontmatter.rs`: Lossless extraction and updating of YAML headers.
- [ ] Implement `src/ops.rs`: Finder reveal commands (`open -R`), system clipboard actions, and safe file deletion (`trash`).

### Phase 2: Slint UI Layout & Reactive Binding

- [ ] Design `ui/appwindow.slint`: 3-pane responsive desktop shell.
- [ ] Create `SlintPaperItem` struct and bind `slint::VecModel` to the center list view.
- [ ] Implement reactive search filtering (matching query against title, authors, tags, and citekey).
- [ ] Implement right-hand Inspector pane: Two-way binding for `status`, `importance` stars, and projects list.

### Phase 3: Ingestion & Stub Creation Flow

- [ ] Detect new PDFs that lack companion notes.
- [ ] Implement "Import All / Import Selected" button:
  * Generates `notes/<basename>.md` with initial YAML frontmatter (`status: unread`).
  * Emits refresh event to reload the list view instantly.

### Phase 4: Windows 10 Metro Adaptation

- [ ] Verify clean compilation on Windows 10 (`cargo build --target x86_64-pc-windows-msvc`).
- [ ] Adapt UI styling parameters for Windows: Apply `Segoe UI`, crisp rectangular corners, and Metro tile cards.
- [ ] Test Explorer reveal command (`explorer /select,`) and normalize path separators (`\` vs `/`).

### Phase 5: Production Packaging

- [ ] Package macOS universal binary and bundle into `.app` / `.dmg`.
- [ ] Package Windows standalone portable executable (`lx-index.exe`).