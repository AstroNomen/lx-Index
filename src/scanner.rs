use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::frontmatter::parse_note_content;
use crate::models::{PaperMetadata, PaperStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct PaperItem {
    pub id: String,
    pub basename: String,
    pub field: String,
    pub pdf_path: Option<PathBuf>,
    pub note_path: Option<PathBuf>,
    pub metadata: PaperMetadata,
    pub has_pdf: bool,
    pub has_note: bool,
    pub is_missing_pdf: bool,
}

#[derive(Clone, Debug, Default)]
pub struct LibraryScanResult {
    pub root: PathBuf,
    pub fields: Vec<String>,
    pub items: Vec<PaperItem>,
    pub projects: Vec<String>,
    pub tags: Vec<String>,
}

/// Scans a library root directory and returns paired literature items and metadata.
pub fn scan_library(root: &Path) -> std::io::Result<LibraryScanResult> {
    if !root.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Library root directory does not exist: {}", root.display()),
        ));
    }

    let mut fields = Vec::new();
    let mut all_items = Vec::new();
    let mut all_projects = BTreeSet::new();
    let mut all_tags = BTreeSet::new();

    // 1. Discover 1st-level directories as Fields
    let entries = fs::read_dir(root)?;
    let mut field_dirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Ignore hidden directories (.git, .lxindex, etc.)
            if dir_name.starts_with('.') {
                continue;
            }

            fields.push(dir_name);
            field_dirs.push(path);
        }
    }

    fields.sort();

    // 2. For each field, collect PDFs and companion notes
    for field_dir in field_dirs {
        let field_name = match field_dir.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let mut pdf_map: HashMap<String, PathBuf> = HashMap::new();
        let mut note_map: HashMap<String, PathBuf> = HashMap::new();

        // Scan PDFs directly under <Field>/
        if let Ok(dir_entries) = fs::read_dir(&field_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext.eq_ignore_ascii_case("pdf") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                pdf_map.insert(stem.to_string(), path);
                            }
                        }
                    }
                }
            }
        }

        // Scan Notes under <Field>/notes/
        let notes_dir = field_dir.join("notes");
        if notes_dir.is_dir() {
            if let Ok(note_entries) = fs::read_dir(&notes_dir) {
                for entry in note_entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if ext.eq_ignore_ascii_case("md") {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    note_map.insert(stem.to_string(), path);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Collect all unique basenames across PDFs and notes
        let mut all_basenames: BTreeSet<String> = BTreeSet::new();
        for k in pdf_map.keys() {
            all_basenames.insert(k.clone());
        }
        for k in note_map.keys() {
            all_basenames.insert(k.clone());
        }

        for basename in all_basenames {
            let maybe_pdf = pdf_map.remove(&basename);
            let maybe_note = note_map.remove(&basename);

            let has_pdf = maybe_pdf.is_some();
            let has_note = maybe_note.is_some();
            let is_missing_pdf = has_note && !has_pdf;

            let id = format!("{}/{}", field_name, basename);

            let metadata = if let Some(ref note_path) = maybe_note {
                if let Ok(content) = fs::read_to_string(note_path) {
                    match parse_note_content(&content) {
                        Ok((meta, _)) => meta,
                        Err(_) => parse_filename_fallback(&basename, &field_name, maybe_pdf.as_deref()),
                    }
                } else {
                    parse_filename_fallback(&basename, &field_name, maybe_pdf.as_deref())
                }
            } else {
                parse_filename_fallback(&basename, &field_name, maybe_pdf.as_deref())
            };

            for p in &metadata.projects {
                all_projects.insert(p.clone());
            }
            for t in &metadata.tags {
                all_tags.insert(t.clone());
            }

            all_items.push(PaperItem {
                id,
                basename,
                field: field_name.clone(),
                pdf_path: maybe_pdf,
                note_path: maybe_note,
                metadata,
                has_pdf,
                has_note,
                is_missing_pdf,
            });
        }
    }

    // Sort items by year descending, then title/basename
    all_items.sort_by(|a, b| {
        b.metadata
            .year
            .cmp(&a.metadata.year)
            .then_with(|| a.basename.cmp(&b.basename))
    });

    Ok(LibraryScanResult {
        root: root.to_path_buf(),
        fields,
        items: all_items,
        projects: all_projects.into_iter().collect(),
        tags: all_tags.into_iter().collect(),
    })
}

/// Parses metadata inferred from standard naming protocol:
/// `[YYYY]_[FirstAuthor]_[TitleKeyword]`
pub fn parse_filename_fallback(
    basename: &str,
    field: &str,
    pdf_path: Option<&Path>,
) -> PaperMetadata {
    let parts: Vec<&str> = basename.split('_').collect();

    let mut year = 0u32;
    let mut authors = Vec::new();
    let mut title = basename.to_string();

    if !parts.is_empty() {
        if let Ok(y) = parts[0].parse::<u32>() {
            if (1900..=2100).contains(&y) {
                year = y;
            }
        }
    }

    if parts.len() >= 2 {
        let author_str = parts[1];
        if !author_str.is_empty() {
            authors.push(author_str.to_string());
        }
    }

    if parts.len() >= 3 {
        let title_words = parts[2..].join(" ");
        title = insert_spaces_between_words(&title_words);
    }

    let citekey = if !authors.is_empty() && year > 0 {
        let short_keyword = parts.get(2).copied().unwrap_or("Paper");
        Some(format!("{}{}{}", authors[0], year, short_keyword))
    } else {
        None
    };

    let pdf_rel_path = pdf_path
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .map(|f| format!("{}/{}", field, f))
        .unwrap_or_default();

    PaperMetadata {
        title,
        citekey,
        doi: None,
        year,
        authors,
        field: field.to_string(),
        projects: Vec::new(),
        status: PaperStatus::Unread,
        importance: 0,
        summary: None,
        tags: Vec::new(),
        pdf_path: pdf_rel_path,
    }
}

/// Helper to convert CamelCase tokens into space-separated words (e.g. "GiantThermoOptic" -> "Giant Thermo Optic")
fn insert_spaces_between_words(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut prev_is_lower = false;

    while let Some(c) = chars.next() {
        if prev_is_lower && c.is_uppercase() {
            result.push(' ');
        }
        prev_is_lower = c.is_lowercase();
        result.push(c);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_parse_filename_fallback() {
        let meta = parse_filename_fallback("2026_Bollani_GiantThermoOptic", "Quantum_Optics", None);
        assert_eq!(meta.year, 2026);
        assert_eq!(meta.authors, vec!["Bollani".to_string()]);
        assert_eq!(meta.title, "Giant Thermo Optic");
        assert_eq!(meta.status, PaperStatus::Unread);
        assert_eq!(meta.citekey, Some("Bollani2026GiantThermoOptic".to_string()));
    }

    #[test]
    fn test_scan_library_end_to_end() {
        let temp_dir = std::env::temp_dir().join("lx_index_test_vault");
        let _ = fs::remove_dir_all(&temp_dir);

        let field1 = temp_dir.join("Quantum_Optics");
        let field1_notes = field1.join("notes");
        fs::create_dir_all(&field1_notes).unwrap();

        // Create paired PDF + Note
        File::create(field1.join("2026_Bollani_GiantThermoOptic.pdf")).unwrap();
        let mut note1 = File::create(field1_notes.join("2026_Bollani_GiantThermoOptic.md")).unwrap();
        write!(
            note1,
            r#"---
title: "Giant thermo-optic tuning"
year: 2026
authors:
  - "Bollani, M."
field: "Quantum_Optics"
projects:
  - "SoftMAX"
status: ai-skimmed
importance: 4
tags:
  - "a-Si"
pdf_path: "Quantum_Optics/2026_Bollani_GiantThermoOptic.pdf"
---
# Notes
"#
        )
        .unwrap();

        // Create orphan PDF (no companion note)
        File::create(field1.join("2025_Shen_OpticalNeuralNet.pdf")).unwrap();

        // Create orphan note (no PDF)
        let mut orphan_note = File::create(field1_notes.join("2024_Solo_NoteOnly.md")).unwrap();
        write!(
            orphan_note,
            r#"---
title: "Note with no PDF"
year: 2024
authors:
  - "Solo, H."
field: "Quantum_Optics"
status: deep-read
importance: 5
tags:
  - "theory"
---
"#
        )
        .unwrap();

        // Also test hidden directory ignoring
        let hidden_dir = temp_dir.join(".lxindex");
        fs::create_dir_all(&hidden_dir).unwrap();

        let scan = scan_library(&temp_dir).unwrap();
        assert_eq!(scan.fields, vec!["Quantum_Optics"]);
        assert_eq!(scan.items.len(), 3);

        // Find paired item
        let paired = scan
            .items
            .iter()
            .find(|i| i.basename == "2026_Bollani_GiantThermoOptic")
            .unwrap();
        assert!(paired.has_pdf);
        assert!(paired.has_note);
        assert!(!paired.is_missing_pdf);
        assert_eq!(paired.metadata.status, PaperStatus::AiSkimmed);
        assert_eq!(paired.metadata.importance, 4);

        // Find orphan PDF
        let orphan_pdf = scan
            .items
            .iter()
            .find(|i| i.basename == "2025_Shen_OpticalNeuralNet")
            .unwrap();
        assert!(orphan_pdf.has_pdf);
        assert!(!orphan_pdf.has_note);
        assert!(!orphan_pdf.is_missing_pdf);
        assert_eq!(orphan_pdf.metadata.status, PaperStatus::Unread);

        // Find orphan note
        let orphan_note_item = scan
            .items
            .iter()
            .find(|i| i.basename == "2024_Solo_NoteOnly")
            .unwrap();
        assert!(!orphan_note_item.has_pdf);
        assert!(orphan_note_item.has_note);
        assert!(orphan_note_item.is_missing_pdf);
        assert_eq!(orphan_note_item.metadata.status, PaperStatus::DeepRead);

        // Check aggregate projects & tags
        assert_eq!(scan.projects, vec!["SoftMAX"]);
        assert!(scan.tags.contains(&"a-Si".to_string()));
        assert!(scan.tags.contains(&"theory".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
