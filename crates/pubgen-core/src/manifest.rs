//! Publication manifest parsing and validation.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

/// Versioned publication manifest (`pubgen_project.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubgenManifest {
    pub version: u32,
    #[serde(rename = "publicationTitle")]
    pub publication_title: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default)]
    pub bilingual: bool,
    #[serde(default)]
    pub document: DocumentSettings,
    #[serde(default)]
    pub chapters: Vec<ChapterEntry>,
    #[serde(default)]
    pub toc: Value,
    #[serde(default)]
    pub build: BuildSettings,
    #[serde(rename = "profileData")]
    pub profile_data: Option<Value>,
    #[serde(rename = "generateSettings")]
    pub generate_settings: Option<Value>,
}

fn default_profile() -> String {
    "bps_brs".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentSettings {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub paper_size: Option<String>,
    #[serde(default)]
    pub margins: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterEntry {
    pub chapter: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub pages: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildSettings {
    #[serde(rename = "exportPdf", default)]
    pub export_pdf: bool,
    #[serde(rename = "outputDocument", default)]
    pub output_document: Option<String>,
    #[serde(rename = "outputPdf", default)]
    pub output_pdf: Option<String>,
}

/// Result of validating a manifest.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl PubgenManifest {
    /// Load and deserialize a manifest from disk.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: invalid manifest JSON: {e}", path.display()))
    }

    /// Validate the manifest integrity and referenced chapter files.
    pub fn validate(&self, base_dir: &Path) -> ValidationReport {
        let mut report = ValidationReport { is_valid: true, errors: Vec::new(), warnings: Vec::new() };

        if self.version == 0 {
            report.errors.push("manifest version must be >= 1".into());
        }

        if self.publication_title.trim().is_empty() {
            report.warnings.push("publication title is empty".into());
        }

        if self.chapters.is_empty() {
            report.errors.push("manifest must contain at least one chapter entry".into());
        }

        for (idx, ch) in self.chapters.iter().enumerate() {
            if ch.path.is_empty() {
                report.warnings.push(format!("chapter {} has no content path specified", ch.chapter));
            } else {
                let full = base_dir.join(&ch.path);
                if !full.is_file() {
                    report.errors.push(format!("chapter {} path not found: {}", ch.chapter, full.display()));
                }
            }
            if ch.chapter != (idx + 1) as u32 {
                report.warnings.push(format!("chapter numbering sequence gap at index {idx}: expected {}, found {}", idx + 1, ch.chapter));
            }
        }

        report.is_valid = report.errors.is_empty();
        report
    }
}
