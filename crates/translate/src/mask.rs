//! Placeholder masking for AI translation: glossary terms and Indonesian-format
//! numbers leave the text as `[[n]]` tokens; `unmask` restores them strictly.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Number,
    Glossary,
}

#[derive(Debug)]
pub struct Token {
    pub index: usize,
    pub original: String,
    pub replacement: Option<String>,
    pub kind: TokenKind,
}

#[derive(Debug)]
pub struct Masked {
    pub text: String,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskError {
    Missing(usize),
    Duplicated(usize),
    Reordered,
}

impl std::fmt::Display for MaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(i) => write!(f, "placeholder [[{i}]] is missing or unknown in the model output"),
            Self::Duplicated(i) => write!(f, "placeholder [[{i}]] appears out of order or duplicated"),
            Self::Reordered => write!(f, "placeholders were reordered by the model"),
        }
    }
}

impl std::error::Error for MaskError {}

/// Tokens per translation unit. Beyond this the rest stays unmasked (long units
/// are `segment()`ed before masking, so hitting the cap means hostile input).
const MAX_TOKENS: usize = 512;

/// Word character for boundary checks (same definition as the glossary domain).
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_number_char(c: char) -> bool {
    c.is_ascii_digit() || c == '.' || c == ','
}

/// Case-insensitive equality of a window against an already-lowercased term.
/// ponytail: compares after `to_lowercase()` of the window; a term whose folding
/// changes char count will not match — fine for Indonesian/ASCII glossaries.
fn window_matches(orig: &[char], start: usize, term_lower: &str) -> bool {
    let count = term_lower.chars().count();
    if start + count > orig.len() {
        return false;
    }
    let window: String = orig[start..start + count].iter().collect();
    window.to_lowercase() == term_lower
}

pub fn mask(text: &str, glossary: &[(String, String)]) -> Masked {
    let chars: Vec<char> = text.chars().collect();
    // Longest term first, so "Luas Wilayah" wins over "Wilayah".
    // Empty or whitespace-only terms (blank CSV rows) are skipped: a zero-length
    // match never advances `i` and would loop forever pushing placeholders.
    let mut terms: Vec<&(String, String)> = glossary.iter().filter(|(t, _)| !t.trim().is_empty()).collect();
    terms.sort_by_key(|(t, _)| std::cmp::Reverse(t.chars().count()));

    let mut tokens: Vec<Token> = Vec::new();
    let mut term_index: Vec<String> = Vec::new(); // lowercase term -> token index lookup
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;

    while i < chars.len() {
        if tokens.len() < MAX_TOKENS {
            // Glossary match (case-insensitive, at word boundaries).
            let mut matched = false;
            for (term, replacement) in &terms {
                let term_lower = term.to_lowercase();
                let len = term_lower.chars().count();
                if window_matches(&chars, i, &term_lower)
                    && (i == 0 || !is_word_char(chars[i - 1]))
                    && (i + len >= chars.len() || !is_word_char(chars[i + len]))
                {
                    let index = match term_index.iter().position(|t| *t == term_lower) {
                        Some(n) => n,
                        None => {
                            term_index.push(term_lower.clone());
                            tokens.push(Token {
                                index: tokens.len(),
                                original: (*term).clone(),
                                replacement: Some(replacement.clone()),
                                kind: TokenKind::Glossary,
                            });
                            tokens.len() - 1
                        }
                    };
                    out.push_str(&format!("[[{index}]]"));
                    i += len;
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
            // Number run: [\d.,]+ with at least one digit, optional trailing '%'.
            if is_number_char(chars[i]) {
                let start = i;
                while i < chars.len() && is_number_char(chars[i]) {
                    i += 1;
                }
                let run: String = chars[start..i].iter().collect();
                let has_percent = i < chars.len() && chars[i] == '%';
                if has_percent {
                    i += 1;
                }
                if run.chars().any(|c| c.is_ascii_digit()) {
                    tokens.push(Token {
                        index: tokens.len(),
                        original: if has_percent { format!("{run}%") } else { run },
                        replacement: None,
                        kind: TokenKind::Number,
                    });
                    out.push_str(&format!("[[{}]]", tokens.len() - 1));
                    continue;
                }
                // No digit (e.g. an ellipsis): keep verbatim.
                out.push_str(&run);
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }

    Masked { text: out, tokens }
}

/// Every `[[digits]]` in `output`, as (byte range, index). Saturating parse:
/// absurd indices stay huge and are rejected as unknown.
fn placeholders(output: &str) -> Vec<(std::ops::Range<usize>, usize)> {
    let bytes = output.as_bytes();
    let mut found = Vec::new();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let mut j = i + 2;
            let mut n = 0usize;
            let mut valid = true;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                n = n.saturating_mul(10).saturating_add((bytes[j] - b'0') as usize);
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b']' && j + 1 < bytes.len() && bytes[j + 1] == b']' {
                // Reject an empty digit run: "[[]]" is not a placeholder.
                if j == i + 2 {
                    valid = false;
                }
                if valid {
                    found.push((i..j + 2, n));
                    i = j + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    found
}

pub fn unmask(output: &str, masked: &Masked) -> Result<String, MaskError> {
    let found = placeholders(output);
    let mut seen = vec![false; masked.tokens.len()];
    let mut last: Option<usize> = None;
    for (_, n) in &found {
        if *n >= masked.tokens.len() {
            // Unknown index (or saturated parse of a hostile digit run).
            return Err(MaskError::Missing(*n));
        }
        if let Some(prev) = last
            && *n < prev
        {
            return Err(if seen[*n] { MaskError::Duplicated(*n) } else { MaskError::Reordered });
        }
        seen[*n] = true;
        last = Some(*n);
    }
    for (i, s) in seen.iter().enumerate() {
        if !s {
            return Err(MaskError::Missing(i));
        }
    }

    let mut out = String::with_capacity(output.len());
    let mut pos = 0usize;
    for (range, n) in &found {
        out.push_str(&output[pos..range.start]);
        let token = &masked.tokens[*n];
        match &token.replacement {
            Some(r) => out.push_str(r),
            None => out.push_str(&token.original),
        }
        pos = range.end;
    }
    out.push_str(&output[pos..]);
    Ok(out)
}

/// Every numeric run in order, `%` stripped — for comparing source and output.
pub fn number_census(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if is_number_char(chars[i]) {
            let start = i;
            while i < chars.len() && is_number_char(chars[i]) {
                i += 1;
            }
            let run: String = chars[start..i].iter().collect();
            if i < chars.len() && chars[i] == '%' {
                i += 1;
            }
            if run.chars().any(|c| c.is_ascii_digit()) {
                out.push(run);
            }
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glossary() -> Vec<(String, String)> {
        vec![("Kecamatan".into(), "Subdistrict".into()), ("Luas Wilayah".into(), "Total Area".into())]
    }

    #[test]
    fn masks_glossary_and_indonesian_numbers() {
        let m = mask("Luas Wilayah Kecamatan Karawang Timur adalah 1.234,56 km2.", &glossary());
        assert!(!m.text.contains("1.234,56"), "the number must not reach the model: {}", m.text);
        assert!(!m.text.contains("Kecamatan"));
        assert!(m.text.contains("[[0]]"));
        let out = m.text.replace("adalah", "is").replace("Km2", "km2");
        // Simulate the model returning the placeholders untouched.
        let restored = unmask(&out, &m).expect("placeholders intact");
        assert!(restored.contains("Subdistrict"), "{restored}");
        assert!(restored.contains("Total Area"), "{restored}");
        assert!(restored.contains("1.234,56"), "{restored}");
    }

    #[test]
    fn longest_glossary_match_wins() {
        let g = vec![("Wilayah".into(), "Region".into()), ("Luas Wilayah".into(), "Total Area".into())];
        let m = mask("Luas Wilayah", &g);
        let restored = unmask(&m.text, &m).expect("intact");
        assert_eq!(restored, "Total Area", "the shorter term must not win");
    }

    #[test]
    fn missing_placeholder_is_an_error_not_a_silent_default() {
        let m = mask("Kecamatan Karawang 1.234,56", &glossary());
        let broken = m.text.replace("[[0]]", "Subdistrict");
        let Err(err) = unmask(&broken, &m) else { panic!("must fail") };
        assert!(matches!(err, MaskError::Missing(_)), "{err}");
    }

    #[test]
    fn an_invented_placeholder_is_an_error() {
        let m = mask("Kecamatan A dan Kecamatan B", &glossary());
        // Both occurrences share one token index by design; a model inventing an
        // extra copy must be caught.
        let broken = format!("{} [[9]]", m.text);
        assert!(unmask(&broken, &m).is_err());
    }

    #[test]
    fn unknown_placeholder_is_an_error() {
        let m = mask("Kecamatan", &glossary());
        let broken = format!("{} [[42]]", m.text);
        assert!(unmask(&broken, &m).is_err());
    }

    #[test]
    fn number_census_preserves_indonesian_format() {
        assert_eq!(number_census("Total 1.234,56 jiwa (2026) 12,5%"), vec!["1.234,56", "2026", "12,5"]);
    }

    #[test]
    fn empty_glossary_and_plain_text_are_unchanged() {
        let m = mask("Halo dunia.", &[]);
        assert_eq!(m.text, "Halo dunia.");
        assert!(m.tokens.is_empty());
        assert_eq!(unmask(&m.text, &m).expect("intact"), "Halo dunia.");
    }

    #[test]
    fn empty_or_whitespace_glossary_terms_are_ignored() {
        // Before the guard the empty term matched at position 0 (first char is
        // non-word) with length 0, so `i` never advanced and mask() looped
        // forever pushing "[[n]]".
        let g = vec![("".into(), "X".into()), ("  ".into(), "Y".into())];
        let m = mask("— Kecamatan", &g);
        assert_eq!(m.text, "— Kecamatan");
        assert!(m.tokens.is_empty());
    }

    #[test]
    fn masking_is_char_boundary_safe() {
        let m = mask("Kecamatan — élève 日本語 1.000", &glossary());
        assert!(unmask(&m.text, &m).is_ok());
    }
}
