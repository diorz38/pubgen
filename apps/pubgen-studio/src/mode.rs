//! Studio perspective modes for PubGen Studio.
//!
//! The visible mode names live in one place, `menus::pubgen_bar` (the pills and the View menu), so this
//! enum stays presentation-free: adding a label here would only create a second, emoji-prefixed spelling
//! of "Orchestrator" / "DesignCraft" (UI consistency F4/F10).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StudioMode {
    /// Publication Orchestrator: Project setup, Excel ingestion, metadata, profiles, TOC, covers.
    #[default]
    Orchestrator,
    /// DesignCraft Layout: Full DTP canvas, typography, optical margins, swatches, and manual overrides.
    DesignCraft,
}
