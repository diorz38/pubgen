//! Structured compile and diagnostic report contracts.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileReport {
    pub status: String,
    #[serde(rename = "compilerVersion")]
    pub compiler_version: String,
    #[serde(rename = "designcraftVersion")]
    pub designcraft_version: String,
    pub pages: usize,
    #[serde(default)]
    pub chapters: Vec<ChapterReport>,
    pub outputs: OutputArtifacts,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<DiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterReport {
    pub chapter: u32,
    pub title: String,
    pub pages: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputArtifacts {
    #[serde(default)]
    pub document: Option<String>,
    #[serde(default)]
    pub pdf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub severity: DiagnosticSeverity,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}
