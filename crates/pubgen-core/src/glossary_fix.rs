//! Deterministic glossary enforcement: the official terminology is applied to a translation
//! even when the model ignored the prompt. Shared by the Orchestrator's translation stage and
//! the pgs app, so neither owns a second copy of the algorithm.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};

/// One vocabulary entry: the Indonesian term and the replacement that must appear instead.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Entry {
    #[serde(default)]
    pub term: String,
    #[serde(default)]
    pub replacement: String,
    #[serde(default)]
    pub do_not_translate: bool,
    #[serde(default)]
    pub case_preserve: bool,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn match_case_of(source: &str) -> bool {
    source.chars().next().is_some_and(|c| c.is_uppercase())
}

fn apply_case(replacement: &str, upper: bool) -> String {
    if !upper {
        return replacement.to_owned();
    }
    replacement
        .split_whitespace()
        .map(|word| match word.chars().next() {
            Some(first) => first.to_uppercase().collect::<String>() + &word[first.len_utf8()..],
            None => word.to_owned(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Replace glossary terms in `text` with their standard replacements at word boundaries.
/// Returns the updated text and the number of replacements applied.
///
/// One left-to-right scan per entry. The scan never looks at replacement text again, so it
/// always terminates — including for an entry whose replacement reproduces its term
/// (`total` → `Total`, `no` → `No`) — and an expanding replacement (`no` → `No data`)
/// cannot grow the text past one substitution per match.
pub fn enforce(text: &str, glossary: &[Entry]) -> (String, usize) {
    let mut fixes = 0usize;
    let mut current = text.to_owned();

    for entry in glossary {
        if entry.term.is_empty() || entry.replacement.is_empty() || entry.do_not_translate {
            continue;
        }
        let needle: Vec<char> = entry.term.to_lowercase().chars().collect();
        if needle.is_empty() {
            continue;
        }

        let chars: Vec<char> = current.chars().collect();
        let lower: Vec<char> = chars.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();
        let mut result = String::with_capacity(current.len());
        let mut entry_fixes = 0usize;
        let mut i = 0usize;
        while i < chars.len() {
            let end = i.saturating_add(needle.len());
            let matched = end <= chars.len()
                && lower[i..end] == needle[..]
                && (i == 0 || !is_word_char(chars[i - 1]))
                && (end == chars.len() || !is_word_char(chars[end]));
            if matched {
                let source: String = chars[i..end].iter().collect();
                let replacement =
                    if entry.case_preserve { apply_case(&entry.replacement, match_case_of(&source)) } else { entry.replacement.clone() };
                result.push_str(&replacement);
                entry_fixes += 1;
                i = end;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        if entry_fixes > 0 {
            fixes += entry_fixes;
            current = result;
        }
    }

    (current, fixes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run `enforce` on a worker thread so a non-terminating scan shows up as a failing
    /// assertion instead of hanging the whole test binary. The threshold is generous: a
    /// correct scan of a few bytes is microseconds.
    fn enforce_with_timeout(text: &str, glossary: &[Entry]) -> (String, usize) {
        let (tx, rx) = std::sync::mpsc::channel();
        let text = text.to_string();
        let glossary = glossary.to_vec();
        std::thread::spawn(move || {
            let _ = tx.send(enforce(&text, &glossary));
        });
        rx.recv_timeout(std::time::Duration::from_secs(5)).expect("glossary enforcement must terminate")
    }

    fn entry(term: &str, replacement: &str, case_preserve: bool) -> Entry {
        Entry { term: term.into(), replacement: replacement.into(), do_not_translate: false, case_preserve }
    }

    #[test]
    fn a_case_preserving_entry_whose_replacement_is_the_needle_terminates() {
        // The bundled glossary has total -> Total: rescanning the text after the replacement
        // matched the same word again, forever. The scan must run once, left to right.
        let (text, fixes) = enforce_with_timeout("Total", &[entry("total", "Total", true)]);
        assert_eq!(text, "Total");
        assert_eq!(fixes, 1);
    }

    #[test]
    fn a_case_preserving_no_entry_terminates() {
        // The bundled glossary also has no -> No, which froze the UI thread on "No."
        let (text, fixes) = enforce_with_timeout("No.", &[entry("no", "No", true)]);
        assert_eq!(text, "No.");
        assert_eq!(fixes, 1);
    }

    #[test]
    fn an_expanding_replacement_is_applied_once_and_never_regrows() {
        // An entry whose replacement contains the term must not be re-scanned: the inserted
        // English is output, not input.
        let (text, fixes) = enforce_with_timeout("no", &[entry("no", "No data", true)]);
        assert_eq!(text, "No data");
        assert_eq!(fixes, 1);
    }

    #[test]
    fn multiple_occurrences_are_all_replaced() {
        let (text, fixes) = enforce_with_timeout("total and total.", &[entry("total", "Total", true)]);
        assert_eq!(text, "Total and Total.");
        assert_eq!(fixes, 2);
    }

    #[test]
    fn test_glossary_enforcement() {
        let glossary = vec![
            Entry { term: "gdp".into(), replacement: "PDB".into(), do_not_translate: false, case_preserve: false },
            Entry { term: "consumer price index".into(), replacement: "indeks harga konsumen".into(), do_not_translate: false, case_preserve: true },
        ];

        let text = "Pertumbuhan GDP kuartal ini dipengaruhi oleh Consumer Price Index yang stabil.";
        let (enforced, count) = enforce(text, &glossary);

        assert_eq!(count, 2);
        assert!(enforced.contains("PDB"));
        assert!(enforced.contains("Indeks Harga Konsumen"));
    }
}
