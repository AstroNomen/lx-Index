<div align="center">
  <img src="assets/icon_256.png" width="128" height="128" alt="lx-Index Logo" />
  <h1>lx-Index</h1>
  <p>Local-first literature management HUD and workspace for academic researchers.</p>

  <p>
    <a href="https://github.com/AstroNomen/lx-Index/releases"><img src="https://img.shields.io/github/v/release/AstroNomen/lx-Index?style=flat-square" alt="Latest Release" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License" /></a>
    <img src="https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=flat-square" alt="Rust Edition" />
    <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey.svg?style=flat-square" alt="Platform Support" />
  </p>
</div>

---

## Overview

lx-Index is a compiled, local-first literature management desktop application. It treats your local filesystem and directory structure as the single source of truth, eliminating proprietary database dependencies and vendor lock-in. Metadata and annotations are synchronized losslessly through standard Markdown YAML frontmatter.

The application delivers platform-native user experiences with decoupled interfaces: an Apple HIG-compliant three-column HUD on macOS and a Modern/Metro Panorama workspace on Windows.

---

## Features

- **Local Filesystem as Single Source of Truth**: Scans folder structures directly. Documents, PDFs, and notes remain standard files on disk without binary database encapsulation.
- **Lossless Markdown Frontmatter Protocol**: Edits metadata (status, importance rating, field, tags, projects, BibTeX) in Markdown frontmatter while keeping personal reading notes and annotations strictly intact.
- **Multi-Select Tag Filtering with In-Flow Auto-Suggestions**: Supports multi-tag intersection (AND logic), frequency-based or alphabetical tag cloud aggregation, and real-time candidate completion during editing.
- **Platform-Tailored Desktop Interfaces**:
  - **macOS**: Native Apple Human Interface Guidelines 3-column continuous layout, Cocoa AppKit window chrome, transparent tall titlebar with concentric traffic lights, native inactive window states, and full system menu bar controls.
  - **Windows**: Windows 8 Metro design language with continuous Pivot columns, bottom App Bar controls, and borderless Aero Snap window handling.
- **Academic Workflow Integration**: One-click BibTeX/Citekey clipboard copying, automatic companion note scaffolding for uncatalogued PDFs, and direct system file manager navigation.

---

## Quick Start & Installation

Pre-compiled binary releases are available on the [Releases](https://github.com/AstroNomen/lx-Index/releases) page.

### macOS (Apple Silicon)

1. Download `lx-index-<version>-macOS.dmg`.
2. Open the disk image and drag `lx Index.app` into your `/Applications` directory.
3. **Gatekeeper Notice**: Because community open-source binaries are not signed with a paid Apple Developer ID certificate, macOS Gatekeeper may display a verification warning on first launch. To bypass:
   - Right-click (or Control-click) `lx Index.app` in `/Applications` and select **Open**; or
   - Run the following terminal command to remove the quarantine flag:
     ```bash
     xattr -cr "/Applications/lx Index.app"
     ```

### Windows (x64)

1. Download `lx-index-<version>-windows-x64.zip`.
2. Extract the archive to any directory.
3. Run `lx-index.exe` directly.
   - The application runs portably without requiring installation or administrative privileges.

### Building from Source

Ensure you have [Rust](https://www.rust-lang.org/) (1.80 or later) installed.

```bash
# Clone repository
git clone https://github.com/AstroNomen/lx-Index.git
cd lx-Index

# Build optimized release binary
cargo build --release
```

The resulting binary will be located at:
- **macOS / Linux**: `target/release/lx-index`
- **Windows**: `target/release/lx-index.exe`

---

## Vault Structure & Metadata Protocol

lx-Index organizes literature into local directories termed **Vaults**. A vault groups papers into academic fields (folders), accompanied by companion Markdown notes and original PDF files.

### Directory Layout

```text
My_Literature_Vault/
├── Condensed_Matter/
│   ├── notes/
│   │   └── 2026_Smith_Superconductivity.md
│   └── 2026_Smith_Superconductivity.pdf
├── Quantum_Optics/
│   ├── notes/
│   │   └── 2025_Doe_Photonics.md
│   └── 2025_Doe_Photonics.pdf
└── Uncatalogued_Paper.pdf
```

### Frontmatter Schema

Each literature note (`notes/<filename>.md`) contains a YAML frontmatter block parsed and written by the application:

```yaml
---
title: "High-Temperature Superconductivity in Twisted Bilayer Graphene"
citekey: "Smith2026Superconductivity"
authors:
  - "Smith, John"
  - "Doe, Jane"
year: 2026
doi: "10.1038/s41586-026-00000-0"
status: 2
importance: 5
field: "Condensed_Matter"
tags:
  - "graphene"
  - "superconductivity"
  - "strongly-correlated"
projects:
  - "QuantumMaterials"
summary: "Demonstrates unconventional pairing mechanisms using angle-resolved photoemission."
bibtex: |
  @article{Smith2026Superconductivity,
    title={High-Temperature Superconductivity in Twisted Bilayer Graphene},
    author={Smith, John and Doe, Jane},
    journal={Nature},
    year={2026}
  }
---

# Reading Notes

Personal reading notes and experimental thoughts start here.
Everything below the closing frontmatter delimiter is preserved byte-for-byte.
```

### Status Codes

| Code | Value | Description |
|---|---|---|
| `0` | Unread | Pending literature backlog |
| `1` | Skimmed | Rapidly reviewed or AI-summarized |
| `2` | Deep-Read | Fully read and digested |
| `3` | Discarded | Filtered out of active review streams |

---

## Configuration

Global settings (registered vault paths, active theme, interface language, and default perspective) are stored in standard operating system configuration paths:

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/lx-index/config.yaml` |
| Windows | `%APPDATA%\lx-index\config.yaml` |
| Linux | `~/.config/lx-index/config.yaml` |

Configuration files are human-readable YAML and can be backed up or transferred across workstations.

---

## Packaging

Automated scripts are provided in the `scripts/` directory for release packaging:

- **macOS**: Generates signed-ready `.app` bundle and compressed UDZO `.dmg` disk image.
  ```bash
  bash scripts/package_macos.sh
  ```
- **Windows**: Compiles release binary, formats user guide, and archives standalone portable `.zip`.
  ```powershell
  powershell -ExecutionPolicy Bypass -File scripts\package_windows.ps1
  ```

---

## Contributing

Contributions are welcomed. Please submit pull requests against the `main` branch. Ensure code changes pass compiler verification and unit tests before submission:

```bash
cargo check
cargo test
```

---

## License

This project is licensed under the [MIT License](LICENSE).
