use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PaperStatus {
    #[default]
    #[serde(alias = "unread", alias = "Unread")]
    Unread,
    #[serde(alias = "ai-skimmed", alias = "ai_skimmed", alias = "AiSkimmed")]
    AiSkimmed,
    #[serde(alias = "deep-read", alias = "deep_read", alias = "DeepRead")]
    DeepRead,
    #[serde(alias = "discarded", alias = "Discarded")]
    Discarded,
}

impl PaperStatus {
    pub fn as_code(&self) -> i32 {
        match self {
            PaperStatus::Unread => 0,
            PaperStatus::AiSkimmed => 1,
            PaperStatus::DeepRead => 2,
            PaperStatus::Discarded => 3,
        }
    }

    pub fn from_code(code: i32) -> Self {
        match code {
            0 => PaperStatus::Unread,
            1 => PaperStatus::AiSkimmed,
            2 => PaperStatus::DeepRead,
            3 => PaperStatus::Discarded,
            _ => PaperStatus::Unread,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PaperStatus::Unread => "unread",
            PaperStatus::AiSkimmed => "ai-skimmed",
            PaperStatus::DeepRead => "deep-read",
            PaperStatus::Discarded => "discarded",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            PaperStatus::Unread => "Unread",
            PaperStatus::AiSkimmed => "AI-Skimmed",
            PaperStatus::DeepRead => "Deep-Read",
            PaperStatus::Discarded => "Discarded",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citekey: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub year: u32,
    #[serde(default)]
    pub authors: Vec<String>,
    pub field: String,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub status: PaperStatus,
    #[serde(default)]
    pub importance: u8, // 1 to 5 stars (0 for none)
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub pdf_path: String,
}

impl Default for PaperMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            citekey: None,
            doi: None,
            year: 0,
            authors: Vec::new(),
            field: String::new(),
            projects: Vec::new(),
            status: PaperStatus::Unread,
            importance: 0,
            tags: Vec::new(),
            summary: None,
            pdf_path: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultRecord {
    pub name: String,
    pub path: String,
}

