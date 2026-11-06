//! `pubgen-core`: Canonical, headless publication compiler and domain engine.
//!
//! Provides UI-independent manifest validation, profile resolution, command planning,
//! diagnostics, and compile orchestration directly on top of DesignCraft engine crates.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod compiler;
pub mod container;
pub mod excel;
pub mod excel_export;
pub mod manifest;
pub mod plan;
pub mod profile;
pub mod report;
pub mod workspace;
pub mod workspace_validation;
pub mod workspace_compile;

pub use compiler::PublicationCompiler;
pub use container::{open_pubgen, save_pubgen, ContainerError, OpenedProject, ProjectContent};
pub use manifest::{BuildSettings, ChapterEntry, DocumentSettings, PubgenManifest, ValidationReport};
pub use plan::{CommandPlan, PublicationOp};
pub use profile::{LayoutGeometry, ProfileId, ProfileService, PublicationProfile};
pub use report::{ChapterReport, CompileReport, DiagnosticItem, DiagnosticSeverity, OutputArtifacts};
pub use workspace::{CardKind, CardScope, Glossary, GlossaryEntry, Language, PageLayout, ProjectManifest, PublicationCard, PublicationMetadata, Slot, PUBGEN_FORMAT_VERSION};
pub use workspace_validation::{ValidationSeverity, WorkspaceDiagnostic, WorkspaceDraft};
pub use excel_export::{export_workbook, ExportError};
