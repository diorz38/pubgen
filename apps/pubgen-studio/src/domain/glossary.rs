//! Glossary enforcement for BPS publications.

use serde::{Deserialize, Serialize};

pub type Glossary = Vec<Entry>;

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

        loop {
            let chars: Vec<char> = current.chars().collect();
            let lower: Vec<char> = chars.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();

            let mut found = None;
            for i in 0..=lower.len().saturating_sub(needle.len()) {
                if lower[i..i + needle.len()] == needle[..]
                    && (i == 0 || !is_word_char(chars[i - 1]))
                    && (i + needle.len() == chars.len() || !is_word_char(chars[i + needle.len()]))
                {
                    found = Some(i);
                    break;
                }
            }

            let Some(at) = found else { break };
            let matched: String = chars[at..at + needle.len()].iter().collect();
            let replacement = if entry.case_preserve { apply_case(&entry.replacement, match_case_of(&matched)) } else { entry.replacement.clone() };

            let mut result: String = chars[..at].iter().collect();
            result.push_str(&replacement);
            result.extend(chars[at + needle.len()..].iter());
            current = result;
            fixes += 1;
        }
    }

    (current, fixes)
}

#[cfg(test)]
mod tests {
    use super::*;

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
