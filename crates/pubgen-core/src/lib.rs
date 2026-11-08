//! `pubgen-core`: Canonical, headless publication compiler and domain engine.
//!
//! Provides UI-independent manifest validation, profile resolution, command planning,
//! diagnostics, and compile orchestration directly on top of DesignCraft engine crates.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod compiler;
pub mod container;
pub mod excel;
pub mod excel_export;
pub mod glossary_fix;
pub mod manifest;
pub mod plan;
pub mod profile;
pub mod report;
pub mod translation;
pub mod translation_units;
pub mod workspace;
pub mod workspace_compile;
pub mod workspace_validation;

pub use compiler::PublicationCompiler;
pub use container::{ContainerError, OpenedProject, ProjectContent, open_pubgen, save_pubgen};
pub use excel_export::{ExportError, export_workbook};
pub use manifest::{BuildSettings, ChapterEntry, DocumentSettings, PubgenManifest, ValidationReport};
pub use plan::{CommandPlan, PublicationOp};
pub use profile::{LayoutGeometry, ProfileId, ProfileService, PublicationProfile};
pub use report::{ChapterReport, CompileReport, DiagnosticItem, DiagnosticSeverity, OutputArtifacts};
pub use translation::{TmEntry, TmOrigin, TranslationMemory, now_stamp};
pub use translation_units::{TransUnit, UnitRole, collect_units, is_skippable, protected_names, split_header_stacks};
pub use workspace::{
    CardKind, CardScope, Glossary, GlossaryEntry, Language, PUBGEN_FORMAT_VERSION, PageLayout, ProjectManifest, PublicationCard, PublicationMetadata,
    Slot,
};
pub use workspace_validation::{ValidationSeverity, WorkspaceDiagnostic, WorkspaceDraft};
