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
    }

    #[cfg(target_os = "windows")]
    {
        let ps_script = r#"[System.Reflection.Assembly]::LoadWithPartialName("System.windows.forms") | Out-Null; $f = New-Object System.Windows.Forms.FolderBrowserDialog; if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $f.SelectedPath }"#;
        let output = std::process::Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(ps_script)
            .output()
            .ok()?;
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    None
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
