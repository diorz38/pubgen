//! Glossary enforcement for BPS publications.
//!
//! The algorithm lives in `pubgen-core` (`glossary_fix`) so the Orchestrator's translation
//! stage and this app share one implementation; this module keeps the app's public path.

// The app's public path stays valid after the algorithm moved to `pubgen-core`; the bin has no
// caller of its own yet, so the re-export is flagged unused here.
#[allow(unused_imports)]
pub use pubgen_core::glossary_fix::{Entry, enforce};

pub type Glossary = Vec<Entry>;
