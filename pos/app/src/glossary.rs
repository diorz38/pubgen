use serde::{Deserialize, Serialize};

pub type Glossary = Vec<Entry>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
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

/// Replace glossary terms in `text` with their replacements, at word boundaries.
/// Returns the new text and the number of fixes applied.
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
            let lower: Vec<char> = chars
                .iter()
                .map(|c| c.to_lowercase().next().unwrap_or(*c))
                .collect();
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
            let replacement = if entry.case_preserve {
                apply_case(&entry.replacement, match_case_of(&matched))
            } else {
                entry.replacement.clone()
            };
            let mut result: String = chars[..at].iter().collect();
            result.push_str(&replacement);
            result.extend(chars[at + needle.len()..].iter());
            current = result;
            fixes += 1;
        }
    }
    (current, fixes)
}

pub fn to_csv(glossary: &Glossary) -> String {
    let mut w = csv::WriterBuilder::new().from_writer(Vec::new());
    let _ = w.write_record(["term", "replacement", "do_not_translate", "case_preserve"]);
    for e in glossary {
        let _ = w.write_record([
            e.term.as_str(),
            e.replacement.as_str(),
            if e.do_not_translate { "true" } else { "" },
            if e.case_preserve { "true" } else { "" },
        ]);
    }
    String::from_utf8(w.into_inner().unwrap_or_default()).unwrap_or_default()
}

pub fn from_csv(text: &str) -> Glossary {
    let mut out = Glossary::new();
    let mut first = true;
    for record in csv::ReaderBuilder::new().from_reader(text.as_bytes()).records() {
        let Ok(record) = record else { continue };
        let get = |i: usize| record.get(i).unwrap_or("").trim().to_owned();
        let (term, replacement, dnt, cp) = (get(0), get(1), get(2), get(3));
        if first {
            first = false;
            if term.eq_ignore_ascii_case("term") {
                continue;
            }
        }
        if term.is_empty() {
            continue;
        }
        out.push(Entry {
            term,
            replacement,
            do_not_translate: dnt.eq_ignore_ascii_case("true"),
            case_preserve: cp.eq_ignore_ascii_case("true"),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(term: &str, replacement: &str, dnt: bool, cp: bool) -> Entry {
        Entry { term: term.into(), replacement: replacement.into(), do_not_translate: dnt, case_preserve: cp }
    }

    #[test]
    fn replaces_with_case_preserve() {
        let g = vec![e("quantum leap", "lompatan kuantum", false, true)];
        let (out, fixes) = enforce("The Quantum Leap story. A quantum leap again.", &g);
        assert_eq!(fixes, 2);
        assert_eq!(out, "The Lompatan Kuantum story. A lompatan kuantum again.");
    }

    #[test]
    fn plain_replacement() {
        let g = vec![e("press", "cetakan", false, false)];
        let (out, fixes) = enforce("The press runs. The Pressroom too.", &g);
        assert_eq!(out, "The cetakan runs. The Pressroom too.");
        assert_eq!(fixes, 1);
    }

    #[test]
    fn dnt_entries_ignored() {
        let g = vec![e("brand", "kept", true, false)];
        let (out, fixes) = enforce("brand stays", &g);
        assert_eq!(out, "brand stays");
        assert_eq!(fixes, 0);
    }

    #[test]
    fn csv_round_trip() {
        let g = vec![e("term a", "repl", false, true), e("brand", "", true, false)];
        let csv = to_csv(&g);
        let back = from_csv(&csv);
        assert_eq!(back, g);
    }

    #[test]
    fn csv_skips_header_and_blank_terms() {
        let csv = "term,replacement\nterm a,repl a\n,broken\n";
        let back = from_csv(csv);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].term, "term a");
    }
}
