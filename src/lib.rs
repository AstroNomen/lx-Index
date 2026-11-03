slint::include_modules!();

pub mod frontmatter;
pub mod i18n;
pub mod models;
pub mod ops;
pub mod scanner;
pub mod state;
pub mod theme;
pub mod vibrancy;
pub mod watcher;

use crate::scanner::PaperItem;

impl PaperItem {
    pub fn to_slint_item(&self) -> SlintPaperItem {
        let projects_display = self.metadata.projects.join(", ");
        let tags_display = self
            .metadata
            .tags
            .iter()
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join(" ");
        let authors = self.metadata.authors.join(", ");

        SlintPaperItem {
            id: self.id.clone().into(),
            filename: self.basename.clone().into(),
            title: self.metadata.title.clone().into(),
            authors: authors.into(),
            year: self.metadata.year as i32,
            field: self.field.clone().into(),
            projects_display: projects_display.into(),
            tags_display: tags_display.into(),
            status_code: self.metadata.status.as_code(),
            importance: self.metadata.importance as i32,
            summary: self.metadata.summary.clone().unwrap_or_default().into(),
            has_pdf: self.has_pdf,
            has_note: self.has_note,
            is_missing_pdf: self.is_missing_pdf,
        }
    }
}
