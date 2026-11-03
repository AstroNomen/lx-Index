use std::fmt;
use crate::models::PaperMetadata;

#[derive(Debug)]
pub enum FrontmatterError {
    NoOpeningDelimiter,
    NoClosingDelimiter,
    YamlError(serde_yaml::Error),
}

impl fmt::Display for FrontmatterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrontmatterError::NoOpeningDelimiter => write!(f, "Missing opening '---' delimiter"),
            FrontmatterError::NoClosingDelimiter => write!(f, "Missing closing '---' delimiter"),
            FrontmatterError::YamlError(e) => write!(f, "YAML error: {}", e),
        }
    }
}

impl std::error::Error for FrontmatterError {}

impl From<serde_yaml::Error> for FrontmatterError {
    fn from(err: serde_yaml::Error) -> Self {
        FrontmatterError::YamlError(err)
    }
}

/// Parses a markdown note into (PaperMetadata, remainder_of_file).
/// The remainder contains everything after the closing delimiter line,
/// preserving every single byte, line ending, and human-written section.
pub fn parse_note_content(content: &str) -> Result<(PaperMetadata, String), FrontmatterError> {
    // Normalizing BOM if present
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    let trimmed_start = content.trim_start();
    if !trimmed_start.starts_with("---") {
        return Err(FrontmatterError::NoOpeningDelimiter);
    }

    // Find the end of the first line
    let first_line_end = match content.find('\n') {
        Some(pos) => pos + 1,
        None => return Err(FrontmatterError::NoClosingDelimiter),
    };

    let rest = &content[first_line_end..];

    // Find the closing '---' or '...' delimiter line
    let mut current_offset = 0;
    let mut closing_start = None;
    let mut remainder_start = None;

    for line in rest.split_inclusive('\n') {
        let line_trimmed = line.trim();
        if line_trimmed == "---" || line_trimmed == "..." {
            closing_start = Some(first_line_end + current_offset);
            remainder_start = Some(first_line_end + current_offset + line.len());
            break;
        }
        current_offset += line.len();
    }

    let (closing_pos, remainder_pos) = match (closing_start, remainder_start) {
        (Some(c), Some(r)) => (c, r),
        _ => return Err(FrontmatterError::NoClosingDelimiter),
    };

    let yaml_slice = &content[first_line_end..closing_pos];
    let remainder = &content[remainder_pos..];

    let metadata: PaperMetadata = serde_yaml::from_str(yaml_slice)?;

    Ok((metadata, remainder.to_string()))
}

/// Updates the YAML frontmatter block while leaving the remainder completely intact.
pub fn update_frontmatter(original_content: &str, metadata: &PaperMetadata) -> Result<String, FrontmatterError> {
    let (_, remainder) = match parse_note_content(original_content) {
        Ok(res) => res,
        Err(FrontmatterError::NoOpeningDelimiter) => {
            // If there's no existing frontmatter, treat the entire original content as remainder
            (PaperMetadata::default(), original_content.to_string())
        }
        Err(e) => return Err(e),
    };

    let mut yaml_str = serde_yaml::to_string(metadata)?;
    if !yaml_str.ends_with('\n') {
        yaml_str.push('\n');
    }

    // Construct the updated note
    let mut result = String::with_capacity(yaml_str.len() + remainder.len() + 16);
    result.push_str("---\n");
    result.push_str(&yaml_str);
    result.push_str("---\n");
    if remainder.starts_with('\n') {
        result.push_str(remainder.trim_start_matches('\n'));
    } else {
        result.push_str(&remainder);
    }

    Ok(result)
}

/// Creates a standardized companion stub note when a new PDF is ingested.
pub fn create_stub_note(metadata: &PaperMetadata) -> String {
    let mut yaml_str = serde_yaml::to_string(metadata).unwrap_or_default();
    if !yaml_str.ends_with('\n') {
        yaml_str.push('\n');
    }

    let bibtex_block = generate_fallback_bibtex(metadata);

    format!(
        "---\n{}---\n\n# {}\n\n## 0. Personal Insights\n- \n\n## 1. Summary\n\n```bibtex\n{}\n```\n",
        yaml_str,
        if metadata.title.is_empty() { "Untitled Document" } else { &metadata.title },
        bibtex_block
    )
}

/// Extracts a ```bibtex ... ``` code block from the companion note markdown if present.
pub fn extract_bibtex_block(content: &str) -> Option<String> {
    let marker_start = "```bibtex";
    let marker_end = "```";

    let start_idx = content.find(marker_start)?;
    let after_start = &content[start_idx + marker_start.len()..];
    let end_idx = after_start.find(marker_end)?;

    let bibtex_content = after_start[..end_idx].trim();
    if bibtex_content.is_empty() {
        None
    } else {
        Some(bibtex_content.to_string())
    }
}

/// Generates a valid BibTeX citation string based on paper metadata.
pub fn generate_fallback_bibtex(metadata: &PaperMetadata) -> String {
    let key = metadata.citekey.clone().unwrap_or_else(|| {
        let first_author = metadata
            .authors
            .first()
            .map(|a| a.split(',').next().unwrap_or(a).trim().replace(' ', ""))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unknown".to_string());
        let short_title = metadata
            .title
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(12)
            .collect::<String>();
        format!("{}{}{}", first_author, metadata.year, short_title)
    });

    let authors_str = if metadata.authors.is_empty() {
        "Unknown".to_string()
    } else {
        metadata.authors.join(" and ")
    };

    let mut fields = Vec::new();
    fields.push(format!("  title = {{{}}}", metadata.title));
    fields.push(format!("  author = {{{}}}", authors_str));
    if metadata.year > 0 {
        fields.push(format!("  year = {{{}}}", metadata.year));
    }
    if let Some(doi) = &metadata.doi {
        if !doi.is_empty() {
            fields.push(format!("  doi = {{{}}}", doi));
        }
    }
    if !metadata.field.is_empty() {
        fields.push(format!("  note = {{Field: {}}}", metadata.field));
    }

    format!("@article{{{},\n{}\n}}", key, fields.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaperStatus;

    #[test]
    fn test_parse_and_lossless_roundtrip() {
        let original_markdown = r#"---
title: "Giant thermo-optic tuning in silicon"
citekey: "Bollani2026Giant"
doi: "10.1038/s41586-026-0001-x"
year: 2026
authors:
  - "Bollani, M."
  - "Shen, Y."
field: "Quantum_Optics"
projects:
  - "SoftMAX"
status: ai-skimmed
importance: 4
tags:
  - "a-Si"
  - "thermo-optic"
pdf_path: "Quantum_Optics/2026_Bollani_GiantThermoOptic.pdf"
---

# 2026_Bollani_GiantThermoOptic

## 0. Personal Insights
- Revolutionary thermo-optic phase shifter with 10x efficiency.
- Compatible with CMOS back-end.

## 1. Method
High temperature sputtering...

```bibtex
@article{Bollani2026Giant,
  title = {Giant thermo-optic tuning in silicon},
  author = {Bollani, M. and Shen, Y.},
  year = {2026}
}
```
"#;

        let (mut meta, remainder) = parse_note_content(original_markdown).expect("Failed to parse note");
        assert_eq!(meta.title, "Giant thermo-optic tuning in silicon");
        assert_eq!(meta.status, PaperStatus::AiSkimmed);
        assert_eq!(meta.importance, 4);
        assert_eq!(meta.year, 2026);
        assert_eq!(meta.authors.len(), 2);
        assert_eq!(meta.projects, vec!["SoftMAX"]);

        assert!(remainder.contains("## 0. Personal Insights"));
        assert!(remainder.contains("Revolutionary thermo-optic phase shifter"));

        // Mutate metadata
        meta.importance = 5;
        meta.status = PaperStatus::DeepRead;
        meta.projects.push("Flatband".to_string());

        let updated_md = update_frontmatter(original_markdown, &meta).expect("Failed to update frontmatter");
        let (re_parsed_meta, re_parsed_remainder) = parse_note_content(&updated_md).expect("Failed to re-parse");

        assert_eq!(re_parsed_meta.importance, 5);
        assert_eq!(re_parsed_meta.status, PaperStatus::DeepRead);
        assert_eq!(re_parsed_meta.projects, vec!["SoftMAX", "Flatband"]);
        assert_eq!(remainder.trim(), re_parsed_remainder.trim());
    }

    #[test]
    fn test_extract_bibtex_block() {
        let content = r#"
Some notes here...

```bibtex
@article{Shen2026,
  title = {Optical Neural Net},
  author = {Shen, Y.}
}
```

Other text
"#;
        let bibtex = extract_bibtex_block(content).expect("Should find bibtex");
        assert!(bibtex.starts_with("@article{Shen2026"));
        assert!(bibtex.ends_with('}'));
    }

    #[test]
    fn test_generate_fallback_bibtex() {
        let meta = PaperMetadata {
            title: "Optical Neural Networks".to_string(),
            citekey: Some("Shen2026Optical".to_string()),
            doi: Some("10.1038/example".to_string()),
            year: 2026,
            authors: vec!["Shen, Y.".to_string(), "Soljacic, M.".to_string()],
            field: "Optical_Analog_Computing".to_string(),
            projects: vec!["SoftMAX".to_string()],
            status: PaperStatus::Unread,
            importance: 3,
            summary: None,
            tags: vec![],
            pdf_path: "path/to/pdf".to_string(),
        };

        let bibtex = generate_fallback_bibtex(&meta);
        assert!(bibtex.contains("@article{Shen2026Optical,"));
        assert!(bibtex.contains("title = {Optical Neural Networks}"));
        assert!(bibtex.contains("author = {Shen, Y. and Soljacic, M.}"));
        assert!(bibtex.contains("year = {2026}"));
        assert!(bibtex.contains("doi = {10.1038/example}"));
    }
}
