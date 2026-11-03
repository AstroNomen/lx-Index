use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::frontmatter::{create_stub_note, update_frontmatter};
use crate::models::{PaperMetadata, PaperStatus, VaultRecord};
use crate::ops::safe_delete_paper;
use crate::scanner::{scan_library, LibraryScanResult, PaperItem};
use crate::theme::AppTheme;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub last_vault_path: Option<String>,
    #[serde(default)]
    pub vaults: Vec<VaultRecord>,
    pub theme_id: Option<String>,
    pub language: Option<String>,
}

pub struct AppState {
    pub vault_path: Option<PathBuf>,
    pub library: LibraryScanResult,
    pub search_query: String,
    pub active_perspective: String, // "all", "unread", "priority", "discarded", "uncatalogued"
    pub active_field: Option<String>,
    pub active_project: Option<String>,
    pub filter_stars: Vec<u8>, // empty = all stars; multi-select list e.g. [5, 4]
    pub selected_id: Option<String>,
    pub theme: AppTheme,
    pub language: String,
    pub registered_vaults: Vec<VaultRecord>,
    pub show_settings: bool,
    pub settings_tab: i32, // 0: Vaults, 1: Theme, 2: Language
}

impl AppState {
    pub fn new() -> Self {
        let (saved_path, saved_vaults, saved_theme_id, saved_lang) = Self::load_config_data();

        let theme = saved_theme_id
            .as_deref()
            .map(AppTheme::by_id)
            .unwrap_or_else(AppTheme::parchment);
        let language = saved_lang.unwrap_or_else(|| "zh".to_string());

        let mut state = Self {
            vault_path: None,
            library: LibraryScanResult::default(),
            search_query: String::new(),
            active_perspective: "all".to_string(),
            active_field: None,
            active_project: None,
            filter_stars: Vec::new(),
            selected_id: None,
            theme,
            language,
            registered_vaults: saved_vaults,
            show_settings: false,
            settings_tab: 0,
        };

        if let Some(path) = saved_path {
            if path.exists() {
                let _ = state.load_vault(&path);
            }
        } else if let Some(first_vault) = state.registered_vaults.first() {
            let path = PathBuf::from(&first_vault.path);
            if path.exists() {
                let _ = state.load_vault(&path);
            }
        }

        state
    }

    /// Loads a vault directory, rescanning all items and updating categories.
    pub fn load_vault(&mut self, path: &Path) -> std::io::Result<()> {
        let scan = scan_library(path)?;
        let path_buf = path.to_path_buf();
        let path_str = path_buf.display().to_string();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Vault")
            .to_string();

        if !self.registered_vaults.iter().any(|v| v.path == path_str) {
            self.registered_vaults.push(VaultRecord {
                name,
                path: path_str,
            });
        }

        self.vault_path = Some(path_buf);
        self.library = scan;
        self.save_current_config();
        Ok(())
    }

    /// Registers a new folder as vault and switches to it.
    pub fn add_vault(&mut self, path: &Path) -> std::io::Result<()> {
        self.load_vault(path)
    }

    /// Unmanages a vault without deleting local disk files.
    pub fn remove_vault(&mut self, path: &str) -> bool {
        let initial_len = self.registered_vaults.len();
        self.registered_vaults.retain(|v| v.path != path);
        let removed = self.registered_vaults.len() < initial_len;

        if removed {
            if let Some(ref current) = self.vault_path {
                if current.display().to_string() == path {
                    if let Some(next_vault) = self.registered_vaults.first() {
                        let next_path = PathBuf::from(&next_vault.path);
                        let _ = self.load_vault(&next_path);
                    } else {
                        self.vault_path = None;
                        self.library = LibraryScanResult::default();
                        self.selected_id = None;
                    }
                }
            }
            self.save_current_config();
        }
        removed
    }

    /// Switches the active vault to one of the registered vaults.
    pub fn switch_vault(&mut self, path: &str) -> std::io::Result<()> {
        let p = PathBuf::from(path);
        if p.exists() {
            self.load_vault(&p)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Vault directory not found: {}", path),
            ))
        }
    }

    /// Sets the active theme by ID.
    pub fn set_theme(&mut self, theme_id: &str) {
        self.theme = AppTheme::by_id(theme_id);
        self.save_current_config();
    }

    /// Sets the active UI language ("zh" or "en").
    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
        self.save_current_config();
    }

    /// Reloads the currently loaded vault if present.
    pub fn reload_current_vault(&mut self) -> std::io::Result<()> {
        if let Some(vault) = self.vault_path.clone() {
            self.load_vault(&vault)?;
        }
        Ok(())
    }

    /// Returns the filtered list of items matching the current active perspective,
    /// field filter, project filter, and search query.
    pub fn filtered_items(&self) -> Vec<&PaperItem> {
        let query = self.search_query.trim().to_lowercase();

        let mut items: Vec<&PaperItem> = self
            .library
            .items
            .iter()
            .filter(|item| {
                // Perspective filter
                match self.active_perspective.as_str() {
                    "uncatalogued" => {
                        if !(item.has_pdf && !item.has_note) {
                            return false;
                        }
                    }
                    "unread" => {
                        if item.metadata.status != PaperStatus::Unread {
                            return false;
                        }
                    }
                    "priority" => {
                        if item.metadata.importance < 4 {
                            return false;
                        }
                    }
                    "discarded" => {
                        if item.metadata.status != PaperStatus::Discarded {
                            return false;
                        }
                    }
                    _ => {
                        // "all": do not show discarded unless in "discarded" perspective
                        if item.metadata.status == PaperStatus::Discarded {
                            return false;
                        }
                    }
                }

                // Field filter
                if let Some(ref field) = self.active_field {
                    if &item.field != field {
                        return false;
                    }
                }

                // Project filter
                if let Some(ref proj) = self.active_project {
                    if !item.metadata.projects.iter().any(|p| p.eq_ignore_ascii_case(proj)) {
                        return false;
                    }
                }

                // Multi-select star rating filter
                if !self.filter_stars.is_empty() && !self.filter_stars.contains(&item.metadata.importance) {
                    return false;
                }

                // Search query filter (matches title, authors, tags, projects, basename, citekey, field, summary)
                if !query.is_empty() {
                    let matches_title = item.metadata.title.to_lowercase().contains(&query);
                    let matches_basename = item.basename.to_lowercase().contains(&query);
                    let matches_summary = item
                        .metadata
                        .summary
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&query))
                        .unwrap_or(false);
                    let matches_authors = item
                        .metadata
                        .authors
                        .iter()
                        .any(|a| a.to_lowercase().contains(&query));
                    let matches_tags = item
                        .metadata
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query));
                    let matches_projects = item
                        .metadata
                        .projects
                        .iter()
                        .any(|p| p.to_lowercase().contains(&query));
                    let matches_citekey = item
                        .metadata
                        .citekey
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&query))
                        .unwrap_or(false);
                    let matches_field = item.field.to_lowercase().contains(&query);

                    if !(matches_title
                        || matches_basename
                        || matches_summary
                        || matches_authors
                        || matches_tags
                        || matches_projects
                        || matches_citekey
                        || matches_field)
                    {
                        return false;
                    }
                }

                true
            })
            .collect();

        // In High Priority perspective, automatically sort by star rating descending (5★ before 4★)
        if self.active_perspective == "priority" {
            items.sort_by(|a, b| {
                b.metadata
                    .importance
                    .cmp(&a.metadata.importance)
                    .then_with(|| b.metadata.year.cmp(&a.metadata.year))
            });
        }

        items
    }

    /// Toggles a star rating filter in the multi-select filter list (0..=5).
    pub fn toggle_filter_star(&mut self, star: u8) {
        if let Some(pos) = self.filter_stars.iter().position(|&s| s == star) {
            self.filter_stars.remove(pos);
        } else {
            self.filter_stars.push(star);
            self.filter_stars.sort_by(|a, b| b.cmp(a));
        }
    }

    /// Clears star filter (all stars shown).
    pub fn clear_filter_stars(&mut self) {
        self.filter_stars.clear();
    }

    /// Selects all stars (0..=5).
    pub fn select_all_filter_stars(&mut self) {
        self.filter_stars = vec![5, 4, 3, 2, 1, 0];
    }

    /// Computes perspective counts across the library:
    /// (count_all, count_unread, count_priority, count_discarded, count_uncatalogued)
    pub fn perspective_counts(&self) -> (usize, usize, usize, usize, usize) {
        let mut count_all = 0;
        let mut count_unread = 0;
        let mut count_priority = 0;
        let mut count_discarded = 0;
        let mut count_uncatalogued = 0;

        for item in &self.library.items {
            if item.has_pdf && !item.has_note {
                count_uncatalogued += 1;
            }

            if item.metadata.status == PaperStatus::Discarded {
                count_discarded += 1;
            } else {
                count_all += 1;
            }

            if item.metadata.status == PaperStatus::Unread {
                count_unread += 1;
            }

            if item.metadata.importance >= 4 {
                count_priority += 1;
            }
        }

        (
            count_all,
            count_unread,
            count_priority,
            count_discarded,
            count_uncatalogued,
        )
    }

    /// Computes item counts per field.
    pub fn field_counts(&self) -> Vec<(String, usize)> {
        let mut map: BTreeMap<String, usize> = BTreeMap::new();
        for f in &self.library.fields {
            map.insert(f.clone(), 0);
        }
        for item in &self.library.items {
            if item.metadata.status != PaperStatus::Discarded {
                *map.entry(item.field.clone()).or_insert(0) += 1;
            }
        }
        map.into_iter().collect()
    }

    /// Computes item counts per project.
    pub fn project_counts(&self) -> Vec<(String, usize)> {
        let mut map: BTreeMap<String, usize> = BTreeMap::new();
        for item in &self.library.items {
            if item.metadata.status != PaperStatus::Discarded {
                for p in &item.metadata.projects {
                    *map.entry(p.clone()).or_insert(0) += 1;
                }
            }
        }
        map.into_iter().collect()
    }

    /// Returns a reference to the selected paper item.
    pub fn selected_item(&self) -> Option<&PaperItem> {
        let sel_id = self.selected_id.as_ref()?;
        self.library.items.iter().find(|i| &i.id == sel_id)
    }

    /// Imports a single uncatalogued paper by generating its companion stub note on disk.
    pub fn import_single_paper(&mut self, id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let vault_root = self.vault_path.clone().ok_or("No vault loaded")?;

        let item = self
            .library
            .items
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or("Paper not found")?;

        let field_dir = vault_root.join(&item.field);
        let notes_dir = field_dir.join("notes");
        fs::create_dir_all(&notes_dir)?;

        let note_path = notes_dir.join(format!("{}.md", item.basename));
        let content = create_stub_note(&item.metadata);
        fs::write(&note_path, content)?;

        item.note_path = Some(note_path.clone());
        item.has_note = true;

        self.recompute_aggregates();
        Ok(note_path)
    }

    /// Ingests all uncatalogued PDFs, generating initial stub notes for each.
    /// Returns the number of newly ingested papers.
    pub fn import_all_uncatalogued(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let vault_root = self.vault_path.clone().ok_or("No vault loaded")?;
        let mut count = 0;

        for item in &mut self.library.items {
            if item.has_pdf && !item.has_note {
                let field_dir = vault_root.join(&item.field);
                let notes_dir = field_dir.join("notes");
                fs::create_dir_all(&notes_dir)?;

                let note_path = notes_dir.join(format!("{}.md", item.basename));
                let content = create_stub_note(&item.metadata);
                fs::write(&note_path, content)?;

                item.note_path = Some(note_path);
                item.has_note = true;
                count += 1;
            }
        }

        if count > 0 {
            self.recompute_aggregates();
        }

        Ok(count)
    }

    /// Mutates the selected paper's metadata and writes to the note file.
    pub fn mutate_selected<F>(&mut self, mutator: F) -> Result<PaperMetadata, Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut PaperMetadata),
    {
        let sel_id = self.selected_id.clone().ok_or("No paper selected")?;
        let vault_root = self.vault_path.clone().ok_or("No vault loaded")?;

        let item = self
            .library
            .items
            .iter_mut()
            .find(|i| i.id == sel_id)
            .ok_or("Selected paper not found in library")?;

        mutator(&mut item.metadata);

        // Discarded workflow: when marking paper Discarded, automatically tag as 2 stars
        if item.metadata.status == PaperStatus::Discarded {
            item.metadata.importance = 2;
        }

        let updated_meta = item.metadata.clone();

        // Write changes to companion note
        if let Some(ref note_path) = item.note_path {
            if note_path.exists() {
                let original_content = fs::read_to_string(note_path)?;
                let new_content = update_frontmatter(&original_content, &updated_meta)?;
                fs::write(note_path, new_content)?;
            } else {
                Self::write_new_note(&vault_root, item, &updated_meta)?;
            }
        } else {
            Self::write_new_note(&vault_root, item, &updated_meta)?;
        }

        // Recompute aggregate projects & tags
        self.recompute_aggregates();

        Ok(updated_meta)
    }

    fn write_new_note(
        vault_root: &Path,
        item: &mut PaperItem,
        metadata: &PaperMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let field_dir = vault_root.join(&item.field);
        let notes_dir = field_dir.join("notes");
        fs::create_dir_all(&notes_dir)?;

        let note_path = notes_dir.join(format!("{}.md", item.basename));
        let content = create_stub_note(metadata);
        fs::write(&note_path, content)?;

        item.note_path = Some(note_path);
        item.has_note = true;
        Ok(())
    }

    /// Deletes the currently selected paper (PDF and/or note) to Trash.
    pub fn delete_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sel_id = self.selected_id.clone().ok_or("No paper selected")?;
        let pos = self
            .library
            .items
            .iter()
            .position(|i| i.id == sel_id)
            .ok_or("Paper not found")?;

        let item = &self.library.items[pos];
        safe_delete_paper(item.pdf_path.as_deref(), item.note_path.as_deref())?;

        self.library.items.remove(pos);
        self.selected_id = None;
        self.recompute_aggregates();
        Ok(())
    }

    fn recompute_aggregates(&mut self) {
        let mut projects = std::collections::BTreeSet::new();
        let mut tags = std::collections::BTreeSet::new();

        for item in &self.library.items {
            for p in &item.metadata.projects {
                projects.insert(p.clone());
            }
            for t in &item.metadata.tags {
                tags.insert(t.clone());
            }
        }

        self.library.projects = projects.into_iter().collect();
        self.library.tags = tags.into_iter().collect();
    }

    // Config persistence
    fn config_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("lx-index").join("config.yaml"))
    }

    pub fn load_config_data() -> (Option<PathBuf>, Vec<VaultRecord>, Option<String>, Option<String>) {
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(config_path) {
                    if let Ok(config) = serde_yaml::from_str::<AppConfig>(&content) {
                        let path = config.last_vault_path.map(PathBuf::from);
                        return (path, config.vaults, config.theme_id, config.language);
                    }
                }
            }
        }
        (None, Vec::new(), None, None)
    }

    pub fn save_current_config(&self) {
        if let Some(config_path) = Self::config_file_path() {
            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let config = AppConfig {
                last_vault_path: self.vault_path.as_ref().map(|p| p.display().to_string()),
                vaults: self.registered_vaults.clone(),
                theme_id: Some(self.theme.id.clone()),
                language: Some(self.language.clone()),
            };
            if let Ok(yaml) = serde_yaml::to_string(&config) {
                let _ = fs::write(config_path, yaml);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_app_state_filtering_and_mutation() {
        let temp_dir = std::env::temp_dir().join("lx_state_test_vault");
        let _ = fs::remove_dir_all(&temp_dir);

        let field = temp_dir.join("Photonics");
        let notes_dir = field.join("notes");
        fs::create_dir_all(&notes_dir).unwrap();

        // 1. Create a paper
        File::create(field.join("2026_Author_LaserCoherent.pdf")).unwrap();
        let mut note = File::create(notes_dir.join("2026_Author_LaserCoherent.md")).unwrap();
        write!(
            note,
            r#"---
title: "Coherent Laser Sources"
citekey: "Author2026Laser"
year: 2026
authors:
  - "Author, A."
field: "Photonics"
projects:
  - "QuantumChip"
status: unread
importance: 3
tags:
  - "laser"
---
# Coherent Laser Sources
"#
        )
        .unwrap();

        let mut state = AppState {
            vault_path: None,
            library: LibraryScanResult::default(),
            search_query: String::new(),
            active_perspective: "all".to_string(),
            active_field: None,
            active_project: None,
            filter_stars: Vec::new(),
            selected_id: None,
            theme: AppTheme::default(),
            language: "zh".to_string(),
            registered_vaults: Vec::new(),
            show_settings: false,
            settings_tab: 0,
        };

        state.load_vault(&temp_dir).unwrap();
        assert_eq!(state.library.items.len(), 1);

        // Test filtering
        assert_eq!(state.filtered_items().len(), 1);

        state.search_query = "Coherent".to_string();
        assert_eq!(state.filtered_items().len(), 1);

        state.search_query = "NonExistent".to_string();
        assert_eq!(state.filtered_items().len(), 0);
        state.search_query.clear();

        // Test selection and mutation
        let id = state.library.items[0].id.clone();
        state.selected_id = Some(id);

        state
            .mutate_selected(|meta| {
                meta.status = PaperStatus::DeepRead;
                meta.importance = 5;
                meta.summary = Some("Personal takeaway notes".to_string());
                meta.projects.push("NextGen".to_string());
            })
            .unwrap();

        let selected = state.selected_item().unwrap();
        assert_eq!(selected.metadata.status, PaperStatus::DeepRead);
        assert_eq!(selected.metadata.importance, 5);
        assert_eq!(selected.metadata.summary.as_deref(), Some("Personal takeaway notes"));
        assert!(selected.metadata.projects.contains(&"NextGen".to_string()));

        // Test summary search matching
        state.search_query = "takeaway".to_string();
        assert_eq!(state.filtered_items().len(), 1);
        state.search_query.clear();

        // Test multi-select star filter
        state.toggle_filter_star(5);
        assert_eq!(state.filtered_items().len(), 1);
        state.toggle_filter_star(5); // untoggle 5 -> empty (all)
        assert_eq!(state.filtered_items().len(), 1);
        state.toggle_filter_star(1); // toggle 1
        assert_eq!(state.filtered_items().len(), 0);
        state.toggle_filter_star(5); // multi-select [5, 1]
        assert_eq!(state.filtered_items().len(), 1);
        state.clear_filter_stars();

        // Test discard auto-importance 2
        state
            .mutate_selected(|meta| {
                meta.status = PaperStatus::Discarded;
            })
            .unwrap();
        let selected_discarded = state.selected_item().unwrap();
        assert_eq!(selected_discarded.metadata.status, PaperStatus::Discarded);
        assert_eq!(selected_discarded.metadata.importance, 2);

        // Check file was actually updated on disk
        let note_content = fs::read_to_string(notes_dir.join("2026_Author_LaserCoherent.md")).unwrap();
        assert!(note_content.contains("status: discarded"));
        assert!(note_content.contains("importance: 2"));
        assert!(note_content.contains("takeaway"));
        assert!(note_content.contains("NextGen"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_app_state_ingestion_and_perspectives() {
        let temp_dir = std::env::temp_dir().join("lx_state_ingest_test_vault");
        let _ = fs::remove_dir_all(&temp_dir);

        let field1 = temp_dir.join("Quantum_Optics");
        let field1_notes = field1.join("notes");
        fs::create_dir_all(&field1_notes).unwrap();

        // 1. Paired item (importance 5, deep-read)
        File::create(field1.join("2026_Bollani_GiantThermoOptic.pdf")).unwrap();
        let mut note1 = File::create(field1_notes.join("2026_Bollani_GiantThermoOptic.md")).unwrap();
        write!(
            note1,
            r#"---
title: "Giant thermo-optic tuning"
citekey: "Bollani2026"
year: 2026
authors:
  - "Bollani, M."
field: "Quantum_Optics"
projects:
  - "SoftMAX"
status: deep-read
importance: 5
tags:
  - "a-Si"
---
# Content
"#
        )
        .unwrap();

        // 2. Uncatalogued PDFs (no companion note yet)
        File::create(field1.join("2026_Shen_OpticalNeuralNet.pdf")).unwrap();
        File::create(field1.join("2025_Bandyopadhyay_Hardware.pdf")).unwrap();

        let mut state = AppState {
            vault_path: None,
            library: LibraryScanResult::default(),
            search_query: String::new(),
            active_perspective: "all".to_string(),
            active_field: None,
            active_project: None,
            filter_stars: Vec::new(),
            selected_id: None,
            theme: AppTheme::default(),
            language: "zh".to_string(),
            registered_vaults: Vec::new(),
            show_settings: false,
            settings_tab: 0,
        };

        state.load_vault(&temp_dir).unwrap();
        assert_eq!(state.library.items.len(), 3);

        // Perspective counts: (all, unread, prio, disc, uncatalogued)
        let (c_all, _, _, _, c_uncatalogued) = state.perspective_counts();
        assert_eq!(c_all, 3);
        assert_eq!(c_uncatalogued, 2);

        // Filter: Uncatalogued
        state.active_perspective = "uncatalogued".to_string();
        let uncat_items = state.filtered_items();
        assert_eq!(uncat_items.len(), 2);

        // Test single ingestion
        let imported_path = state
            .import_single_paper("Quantum_Optics/2026_Shen_OpticalNeuralNet")
            .unwrap();
        assert!(imported_path.exists());

        // Note now exists
        let note_content = fs::read_to_string(&imported_path).unwrap();
        assert!(note_content.contains("status: unread"));
        assert!(note_content.contains("## 0. Personal Insights"));

        // Uncatalogued count now 1
        let (_, _, _, _, c_after_single) = state.perspective_counts();
        assert_eq!(c_after_single, 1);

        // Test batch ingestion of remaining uncatalogued papers
        let imported_count = state.import_all_uncatalogued().unwrap();
        assert_eq!(imported_count, 1);

        // Uncatalogued count now 0
        let (_, _, _, _, c_after_all) = state.perspective_counts();
        assert_eq!(c_after_all, 0);

        let note2_path = field1_notes.join("2025_Bandyopadhyay_Hardware.md");
        assert!(note2_path.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
