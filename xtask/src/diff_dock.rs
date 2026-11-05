//! `cargo xtask diff-dock`: checks whether `crates/ui-egui/src/dock.rs` (upstream baseline)
//! and `crates/ui-egui/src/dock_extended.rs` have drifted outside of the known Pubgen hooks.
//!
//! The extended file is the upstream baseline plus a handful of Pubgen-only insertions (the
//! `pubgen` module, the `PUBGEN_PANELS` const, the `pubgen_menu`/`pubgen_dialogs` wrappers, and
//! the extra `pubgen_menu_tests` module). After stripping those hooks, every upstream line must
//! still appear, in order, in the extended file — i.e. the upstream content is a subsequence.
//! That tolerates insertions (the normal case) while still catching drift: a deleted, reordered or
//! edited upstream line breaks the subsequence.

use std::path::Path;

/// Lines that exist only in the Pubgen fork and have no upstream counterpart.
const HOOK_PATTERNS: &[&str] = &[
    "pub mod pubgen;",
    "panels/pubgen",
    "pub const PUBGEN_PANELS",
    "/// The dock panels the Pubgen fork adds",
    r#""pubgen" => pubgen::show(app, ui),"#,
    "/// The Pubgen top-level application menu (shown in the main menu bar).",
    "/// Its rows live in [`crate::menus::pubgen_bar`], beside the PubGen menu bar.",
    "pub fn pubgen_menu(app: &mut DesignApp, ui: &mut egui::Ui) {",
    "crate::menus::pubgen_bar::pubgen_menu(app, ui);",
    "/// Global dialogs and background tasks handler for Pubgen.",
    "pub fn pubgen_dialogs(app: &mut DesignApp, ctx: &egui::Context) {",
    "pubgen::pubgen_dialogs(app, ctx);",
];

/// Substring replacements that turn a Pubgen-only line into its upstream form (or remove it).
const HOOK_REPLACEMENTS: &[(&str, &str)] = &[
    (r#", ("pubgen", "Pubgen", "panel-pages")"#, ""),
    (r#"("pubgen", "Pubgen", "panel-pages"),"#, ""),
    (r#"("pubgen", "Pubgen", "panel-pages")"#, ""),
    (".chain(PUBGEN_PANELS)", ""),
];

pub fn run(root: &Path) -> Result<(), String> {
    let dock_path = root.join("crates/ui-egui/src/dock.rs");
    let extended_path = root.join("crates/ui-egui/src/dock_extended.rs");

    if !dock_path.exists() {
        return Err(format!("dock.rs not found at {}", dock_path.display()));
    }
    if !extended_path.exists() {
        return Err(format!("dock_extended.rs not found at {}", extended_path.display()));
    }

    let dock_raw = std::fs::read_to_string(&dock_path).map_err(|e| format!("read dock.rs: {e}"))?;
    let ext_raw = std::fs::read_to_string(&extended_path).map_err(|e| format!("read dock_extended.rs: {e}"))?;

    let dock_lines = clean_lines(&dock_raw, false);
    let ext_lines = clean_lines(&ext_raw, true);

    if is_subsequence(&dock_lines, &ext_lines) {
        println!("✓ dock.rs and dock_extended.rs are in sync (Pubgen hooks intact).");
        Ok(())
    } else {
        // Find the first upstream line that has no match in the extended file, in order, so the
        // drift is reported at a useful location rather than as a raw diff.
        let mut ext_iter = ext_lines.iter().peekable();
        let mut first_missing: Option<(usize, &str)> = None;
        let mut missing = 0usize;
        for (i, line) in dock_lines.iter().enumerate() {
            while ext_iter.peek().is_some_and(|e| *e != line) {
                ext_iter.next();
            }
            if ext_iter.peek().is_none() {
                missing += 1;
                if first_missing.is_none() {
                    first_missing = Some((i, line));
                }
            }
        }
        println!("! Drift detected between dock.rs and dock_extended.rs:\n");
        if let Some((i, line)) = first_missing {
            println!("  first upstream line with no matching extended line:");
            println!("    upstream line {}: {line}", i + 1);
        }
        println!("  {missing} upstream line(s) have no counterpart in dock_extended.rs");
        println!("\nRun reconciliation: merge upstream changes from dock.rs into dock_extended.rs.");
        Err(format!("dock.rs and dock_extended.rs differ ({missing} lines)"))
    }
}

/// Normalize lines: trim, apply hook substring replacements, drop hook-only lines, drop blanks.
fn clean_lines(content: &str, strip_hooks: bool) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .map(
            |line| {
                if strip_hooks {
                    HOOK_REPLACEMENTS.iter().fold(line.to_string(), |line, (from, to)| line.replace(from, to))
                } else {
                    line.to_string()
                }
            },
        )
        .filter(|line| {
            if line.is_empty() {
                return false;
            }
            if strip_hooks && HOOK_PATTERNS.iter().any(|p| line.contains(p)) {
                return false;
            }
            true
        })
        .collect()
}

/// Is `needle` a subsequence of `haystack`? Both are already normalized.
fn is_subsequence(needle: &[String], haystack: &[String]) -> bool {
    let mut i = 0usize;
    for line in haystack {
        if i < needle.len() && *line == needle[i] {
            i += 1;
            if i == needle.len() {
                return true;
            }
        }
    }
    i == needle.len()
}
