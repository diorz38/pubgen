//! Command Plan representation for publication layout operations.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublicationOp {
    CreateCover { title: String, subtitle: String, release_no: String, agency: String },
    CreateDivider { chapter: u32, title: String },
    InsertStory { source: PathBuf, autoflow: bool },
    InsertTable { source: PathBuf, autoflow: bool },
    InsertChart { spec_json: String },
    GenerateToc { title: String, style: String },
    ApplyHeaderFooter { header_text: String, footer_text: String },
    ExportPdf { destination: PathBuf },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandPlan {
    pub ops: Vec<PublicationOp>,
}

impl CommandPlan {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn push(&mut self, op: PublicationOp) {
        self.ops.push(op);
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}
