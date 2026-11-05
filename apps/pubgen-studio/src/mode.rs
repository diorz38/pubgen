//! Studio perspective modes for Pubgen Studio.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StudioMode {
    /// Publication Orchestrator: Project setup, Excel ingestion, metadata, profiles, TOC, covers.
    #[default]
    Orchestrator,
    /// DesignCraft Layout: Full DTP canvas, typography, optical margins, swatches, and manual overrides.
    DesignCraft,
}

impl StudioMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Orchestrator => "📊 Orchestrator",
            Self::DesignCraft => "🎨 DesignCraft Layout",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Orchestrator => "Publication data ingestion, profiles, metadata, and automated compiler",
            Self::DesignCraft => "Interactive DTP layout canvas, typography, optical alignment, and export",
        }
    }
}

