//! Persistent translation memory: reviewed terms survive across sessions.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Guard: packages are untrusted input; a corrupt or hostile TM file must not grow the memory
/// (and thus save/open cost) without bound.
pub const MAX_ENTRIES: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TmOrigin {
    Ai,
    Reviewed,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmEntry {
    pub source: String,
    pub target: String,
    pub origin: TmOrigin,
    pub role: String,
    pub updated: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationMemory {
    pub entries: Vec<TmEntry>,
}

/// Trim plus collapse internal whitespace runs to one space, the key every lookup and insert uses.
fn normalize(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl TranslationMemory {
    pub fn lookup(&self, source: &str) -> Option<&TmEntry> {
        let key = normalize(source);
        self.find(&key, false).or_else(|| self.find(&key.to_lowercase(), true))
    }
    fn find(&self, key: &str, case_insensitive: bool) -> Option<&TmEntry> {
        let mut best: Option<&TmEntry> = None;
        for entry in &self.entries {
            let candidate = normalize(&entry.source);
            let same = if case_insensitive { candidate.to_lowercase() == key } else { candidate == key };
            let better = match best {
                None => true,
                Some(b) => matches!(b.origin, TmOrigin::Ai) && !matches!(entry.origin, TmOrigin::Ai),
            };
            if same && better {
                best = Some(entry);
            }
        }
        best
    }
    pub fn insert(&mut self, entry: TmEntry) -> bool {
        let key = normalize(&entry.source);
        if key.is_empty() || entry.target.trim().is_empty() {
            return false;
        }
        if let Some(index) = self.entries.iter().position(|e| normalize(&e.source) == key) {
            let existing = &self.entries[index];
            if matches!(existing.origin, TmOrigin::Reviewed | TmOrigin::Imported) && matches!(entry.origin, TmOrigin::Ai) {
                return false; // Review is a gate: AI must never clobber a human-reviewed term.
            }
            self.entries[index] = entry;
            return true;
        }
        if self.entries.len() >= MAX_ENTRIES {
            return false;
        }
        self.entries.push(entry);
        true
    }
    pub fn promote(&mut self, source: &str) -> bool {
        let key = normalize(source);
        let Some(entry) = self.entries.iter_mut().find(|e| normalize(&e.source) == key) else { return false };
        if entry.origin != TmOrigin::Ai {
            return false;
        }
        entry.origin = TmOrigin::Reviewed;
        entry.updated = now_stamp();
        true
    }
    pub fn update_target(&mut self, source: &str, target: &str) -> bool {
        let key = normalize(source);
        let Some(entry) = self.entries.iter_mut().find(|e| normalize(&e.source) == key) else { return false };
        entry.target = target.to_owned();
        entry.origin = TmOrigin::Reviewed;
        entry.updated = now_stamp();
        true
    }
    /// Drop the entry `lookup` would return (same whitespace normalization, then the same
    /// case-insensitive fallback). Returns whether anything was removed: the Reject button.
    pub fn remove(&mut self, source: &str) -> bool {
        let key = normalize(source);
        if key.is_empty() {
            return false;
        }
        let index = self.entries.iter().position(|e| normalize(&e.source) == key).or_else(|| {
            let lower = key.to_lowercase();
            self.entries.iter().position(|e| normalize(&e.source).to_lowercase() == lower)
        });
        match index {
            Some(i) => {
                self.entries.remove(i);
                true
            }
            None => false,
        }
    }
    pub fn pending_review(&self) -> Vec<&TmEntry> {
        self.entries.iter().filter(|e| e.origin == TmOrigin::Ai).collect()
    }
    /// Keep at most [`MAX_ENTRIES`] entries. Called when a package's `translation_memory.json`
    /// is loaded: the file is untrusted input, and `insert` alone cannot shrink an already
    /// oversized memory read from disk.
    pub fn truncate_to_cap(&mut self) {
        self.entries.truncate(MAX_ENTRIES);
    }
    pub fn to_map(&self) -> BTreeMap<String, String> {
        // `insert` rejects a whitespace-only target; a memory loaded from disk may still carry
        // one, and the generation cache must never pair an empty English under the Indonesian.
        self.entries.iter().filter(|e| !e.target.trim().is_empty()).map(|e| (normalize(&e.source), e.target.clone())).collect()
    }
}

pub fn now_stamp() -> String {
    // chrono in this workspace carries no `clock` feature, so the wall clock comes from std and
    // chrono only formats it (house rule: calendar math goes through chrono).
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => chrono::DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos()).map(|t| t.to_rfc3339()).unwrap_or_default(),
        Err(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(src: &str, tgt: &str, origin: TmOrigin) -> TmEntry {
        TmEntry { source: src.into(), target: tgt.into(), origin, role: "table_title".into(), updated: now_stamp() }
    }

    #[test]
    fn lookup_is_whitespace_normalized() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Luas  Wilayah", "Total Area", TmOrigin::Reviewed));
        assert_eq!(tm.lookup("Luas Wilayah").map(|e| e.target.as_str()), Some("Total Area"));
    }

    #[test]
    fn a_reviewed_entry_is_never_overwritten_by_ai() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Kecamatan", "Subdistrict", TmOrigin::Reviewed));
        let changed = tm.insert(entry("Kecamatan", "District", TmOrigin::Ai));
        assert!(!changed, "AI must not clobber a reviewed term");
        assert_eq!(tm.lookup("Kecamatan").map(|e| e.target.as_str()), Some("Subdistrict"));
    }

    #[test]
    fn ai_may_refresh_an_older_ai_entry() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Kecamatan", "District", TmOrigin::Ai));
        assert!(tm.insert(entry("Kecamatan", "Subdistrict", TmOrigin::Ai)));
        assert_eq!(tm.lookup("Kecamatan").map(|e| e.target.as_str()), Some("Subdistrict"));
    }

    #[test]
    fn promote_moves_ai_to_reviewed() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("PDRB", "GRDP", TmOrigin::Ai));
        assert_eq!(tm.pending_review().len(), 1);
        assert!(tm.promote("PDRB"));
        assert!(tm.pending_review().is_empty());
        assert!(!tm.promote("PDRB"), "promoting twice is a no-op");
    }

    #[test]
    fn update_target_marks_the_entry_reviewed() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("PDRB", "GDP", TmOrigin::Ai));
        assert!(tm.update_target("PDRB", "GRDP"));
        assert_eq!(tm.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
    }

    #[test]
    fn remove_drops_the_entry_lookup_would_find() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("PDRB", "GRDP", TmOrigin::Ai));
        // Same keys `lookup` accepts: collapsed whitespace, then case-insensitive.
        assert!(tm.remove("  pdrb "));
        assert!(tm.lookup("PDRB").is_none());
        assert!(tm.entries.is_empty());
        assert!(!tm.remove("PDRB"), "removing a missing entry is a no-op");
    }

    #[test]
    fn to_map_feeds_the_generation_cache() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Kecamatan", "Subdistrict", TmOrigin::Ai));
        tm.insert(entry("PDRB", "GRDP", TmOrigin::Reviewed));
        let map = tm.to_map();
        assert_eq!(map.get("Kecamatan").map(String::as_str), Some("Subdistrict"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn to_map_skips_a_whitespace_only_target() {
        // `insert` refuses such a target, but a memory loaded from disk may carry one and the
        // generation cache must not pair an empty English.
        let tm = TranslationMemory {
            entries: vec![entry("Kecamatan", "Subdistrict", TmOrigin::Ai), entry("Desa", "   ", TmOrigin::Ai), entry("Kelurahan", "", TmOrigin::Ai)],
        };
        let map = tm.to_map();
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("Kecamatan").map(String::as_str), Some("Subdistrict"));
        assert!(map.get("Desa").is_none(), "whitespace-only target leaked into the cache");
    }

    #[test]
    fn truncate_to_cap_keeps_the_first_entries() {
        // A hostile package can deserialize more entries than `insert` would ever accept.
        let mut tm = TranslationMemory { entries: (0..=MAX_ENTRIES).map(|i| entry(&format!("s{i}"), "t", TmOrigin::Ai)).collect() };
        assert_eq!(tm.entries.len(), MAX_ENTRIES + 1);
        tm.truncate_to_cap();
        assert_eq!(tm.entries.len(), MAX_ENTRIES);
        assert_eq!(tm.entries.last().map(|e| e.source.as_str()), Some(format!("s{}", MAX_ENTRIES - 1).as_str()));
    }

    #[test]
    fn empty_source_is_rejected() {
        let mut tm = TranslationMemory::default();
        assert!(!tm.insert(entry("   ", "x", TmOrigin::Ai)));
        assert!(tm.entries.is_empty());
    }

    #[test]
    fn insert_is_capped_at_100_000_entries() {
        // Guard: an untrusted TM file must not grow the memory without bound.
        let mut tm = TranslationMemory { entries: (0..100_000).map(|i| entry(&format!("s{i}"), "t", TmOrigin::Ai)).collect() };
        assert!(!tm.insert(entry("one more", "t", TmOrigin::Ai)));
        assert_eq!(tm.entries.len(), 100_000);
        // Replacing an existing entry still works at the cap.
        assert!(tm.insert(entry("s0", "replaced", TmOrigin::Ai)));
    }
}
