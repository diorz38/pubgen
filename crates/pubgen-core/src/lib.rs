//! `pubgen-core`: Canonical, headless publication compiler and domain engine.
//!
//! Provides UI-independent manifest validation, profile resolution, command planning,
//! diagnostics, and compile orchestration directly on top of DesignCraft engine crates.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod compiler;
pub mod manifest;
pub mod plan;
pub mod profile;
pub mod report;

pub use compiler::PublicationCompiler;
pub use manifest::{BuildSettings, ChapterEntry, DocumentSettings, PubgenManifest, ValidationReport};
pub use plan::{CommandPlan, PublicationOp};
pub use profile::{LayoutGeometry, ProfileId, ProfileService, PublicationProfile};
pub use report::{ChapterReport, CompileReport, DiagnosticItem, DiagnosticSeverity, OutputArtifacts};
