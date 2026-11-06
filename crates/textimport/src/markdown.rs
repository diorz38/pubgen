//! Markdown → story: headings, emphasis, lists, block quotes and hard line breaks become a
//! formatted [`Story`] (File › Place of `.md`). No paragraph styles are invented: headings carry
//! local bold and size, quotes local italic, so the text lands clean in any template.

use designcraft_doc::story::{BASIC_PARAGRAPH, NO_CHAR_STYLE};
use designcraft_doc::{CharAttrs, CharFormat, ParaAttrs, ParaFormat, Story, StoryId};

use crate::Imported;

/// Heading sizes, `#` through `######`.
const HEADING_SIZES: [f64; 6] = [24.0, 20.0, 17.0, 15.0, 13.0, 12.0];

/// A run's character state: bold/italic flags and an optional local size (0.0 = none).
#[derive(Clone, Copy)]
struct Run {
    bold: bool,
    italic: bool,
    size: f64,
}

const PLAIN: Run = Run { bold: false, italic: false, size: 0.0 };

fn fmt(r: Run) -> CharFormat {
    CharFormat {
        style: NO_CHAR_STYLE.into(),
        over: CharAttrs {
            font_style: match (r.bold, r.italic) {
                (true, true) => Some("Bold Italic".into()),
                (true, false) => Some("Bold".into()),
                (false, true) => Some("Italic".into()),
                _ => None,
            },
            size: (r.size > 0.0).then_some(r.size),
            ..Default::default()
        },
    }
}

/// Starts a paragraph (paragraph break before it unless it is the first) with `attrs`.
fn start_para(st: &mut Story, first: &mut bool, attrs: ParaAttrs) {
    if !*first {
        let len = st.len();
        st.insert(len, "\n");
    }
    *first = false;
    let at = st.len();
    st.format_paras(at..at, |f| *f = ParaFormat { style: BASIC_PARAGRAPH.into(), para: attrs.clone(), ..Default::default() });
}

pub fn import(text: &str) -> Imported {
    let t = text.trim_start_matches('\u{FEFF}').replace("\r\n", "\n").replace('\r', "\n");
    let mut st = Story::new(StoryId(0));
    let mut first = true;
    for block in split_blocks(&t) {
        let lines: Vec<&str> = block.split('\n').collect();
        match classify(&lines) {
            (Block::Heading(level), body) => {
                let size = HEADING_SIZES[level.saturating_sub(1).min(5)];
                start_para(&mut st, &mut first, ParaAttrs::default());
                inline(&mut st, &body, Run { bold: true, size, ..PLAIN });
            }
            (Block::Quote, body) => {
                start_para(&mut st, &mut first, ParaAttrs { left_indent: Some(18.0), ..Default::default() });
                inline(&mut st, &body, Run { italic: true, ..PLAIN });
            }
            (Block::List { bullet }, _) => {
                for line in &lines {
                    let rest = line.trim_start().strip_prefix("- ").or_else(|| line.trim_start().strip_prefix("* ")).unwrap_or(line.trim_start());
                    start_para(&mut st, &mut first, ParaAttrs { left_indent: Some(14.0), ..Default::default() });
                    let text = if bullet { format!("• {rest}") } else { rest.to_string() };
                    inline(&mut st, &text, PLAIN);
                }
            }
            (Block::Code, _) => {
                for line in &lines[1..lines.len().saturating_sub(1)] {
                    start_para(&mut st, &mut first, ParaAttrs::default());
                    inline(&mut st, line, PLAIN);
                }
            }
            (Block::Plain, body) => {
                start_para(&mut st, &mut first, ParaAttrs::default());
                inline(&mut st, &body, PLAIN);
            }
        }
    }
    Imported { story: st, para_styles: vec![], char_styles: vec![], warnings: vec![] }
}

enum Block {
    Heading(usize),
    Quote,
    /// `bullet` distinguishes `-`/`*` markers (rendered as `• `) from `1. ` numbering (kept as
    /// written). ponytail: real list attrs (numbering) when the engine imports lists from md.
    List {
        bullet: bool,
    },
    Code,
    Plain,
}

/// Paragraphs of the text: blocks separated by blank lines (a fenced ``` block is one block).
fn split_blocks(t: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_fence = false;
    for line in t.split('\n') {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if line.trim().is_empty() && !in_fence {
            if !cur.trim().is_empty() {
                out.push(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
            continue;
        }
        if !cur.is_empty() {
            cur.push('\n');
        }
        cur.push_str(line);
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

fn classify(lines: &[&str]) -> (Block, String) {
    let first_line = lines.first().copied().unwrap_or_default();
    let hashes = first_line.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&hashes) && first_line.get(hashes..hashes + 1) == Some(" ") {
        return (Block::Heading(hashes), first_line.get(hashes + 1..).unwrap_or_default().trim().to_string());
    }
    if !lines.is_empty() && lines.iter().all(|l| l.starts_with('>')) {
        let body = lines.iter().map(|l| l.strip_prefix('>').unwrap_or(l).strip_prefix(' ').unwrap_or(l)).collect::<Vec<_>>().join(" ");
        return (Block::Quote, body);
    }
    if !lines.is_empty() && lines.iter().all(|l| l.trim_start().starts_with("- ") || l.trim_start().starts_with("* ")) {
        return (Block::List { bullet: true }, String::new());
    }
    if !lines.is_empty() && lines.iter().all(|l| numbered(l)) {
        return (Block::List { bullet: false }, String::new());
    }
    if first_line.trim_start().starts_with("```") {
        return (Block::Code, String::new());
    }
    // A soft-wrapped paragraph joins its lines with spaces; a trailing two spaces (or backslash)
    // is a hard break instead.
    let mut body = String::new();
    for line in lines {
        let hard_break_before = body.ends_with(designcraft_doc::FORCED_LINE_BREAK);
        if !body.is_empty() && !hard_break_before {
            body.push(' ');
        }
        let hard = line.ends_with("  ") || line.ends_with('\\');
        body.push_str(line.trim_end_matches([' ', '\\']));
        if hard {
            body.push(designcraft_doc::FORCED_LINE_BREAK);
        }
    }
    (Block::Plain, body)
}

fn numbered(line: &str) -> bool {
    let l = line.trim_start();
    let digits = l.chars().take_while(char::is_ascii_digit).count();
    digits >= 1 && l.get(digits..digits + 2) == Some(". ")
}

/// Appends `s` to `st` as runs, resolving `***`/`**`/`*`/`_` emphasis and `` ` `` code spans.
fn inline(st: &mut Story, s: &str, base: Run) {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    let mut run = String::new();
    while i < chars.len() {
        let c = chars[i];
        if c == '`'
            && let Some(close) = chars[i + 1..].iter().position(|&x| x == '`')
        {
            push(st, &mut run, PLAIN);
            let code: String = chars[i + 1..i + 1 + close].iter().collect();
            run.push_str(&code);
            push(st, &mut run, PLAIN);
            i += close + 2;
            continue;
        }
        if c == '*' || c == '_' {
            let stars = chars[i..].iter().take_while(|&&x| x == c).count().min(3);
            let marker: Vec<char> = chars[i..i + stars].to_vec();
            if let Some(found) = find_close(&chars[i + stars..], &marker) {
                push(st, &mut run, base);
                let inner: String = chars[i + stars..i + stars + found].iter().collect();
                let extra = match stars {
                    3 => Run { bold: true, italic: true, ..base },
                    2 => Run { bold: true, ..base },
                    _ => Run { italic: true, ..base },
                };
                inline(st, &inner, extra);
                i += stars + found + stars;
                continue;
            }
        }
        run.push(c);
        i += 1;
    }
    push(st, &mut run, base);
}

/// Where the emphasis marker closes: not at the very start, and not right after whitespace
/// (a marker against a space on its left is literal, not a closer).
fn find_close(chars: &[char], marker: &[char]) -> Option<usize> {
    let n = marker.len();
    let mut i = 0usize;
    while i + n <= chars.len() {
        if chars[i..i + n] == *marker && i > 0 && !chars[i - 1].is_whitespace() {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn push(st: &mut Story, run: &mut String, r: Run) {
    if run.is_empty() {
        return;
    }
    let len = st.len();
    st.insert_with(len, run, fmt(r));
    run.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The texts of every run whose attributes match.
    fn run_texts(st: &Story, want: &CharFormat) -> Vec<String> {
        let mut at = 0usize;
        let mut out = Vec::new();
        for r in &st.chars {
            if &r.format == want
                && let Some(t) = st.text.get(at..at + r.len)
            {
                out.push(t.trim_end_matches('\n').to_string());
            }
            at += r.len;
        }
        out
    }

    #[test]
    fn headings_emphasis_lists_and_quotes() {
        let i = import("# Judul\n\nParagraf dengan **tebal**, _miring_ dan `kode`.\n\n- satu\n- dua\n\n> kutipan\n\n1. pertama");
        let st = &i.story;
        assert_eq!(st.text, "Judul\nParagraf dengan tebal, miring dan kode.\n• satu\n• dua\nkutipan\n1. pertama");
        assert_eq!(st.paras.len(), 6);
        // The heading is bold at 24 pt; **tebal** is bold; `kode` is plain.
        assert!(run_texts(st, &fmt(Run { bold: true, size: 24.0, ..PLAIN })).iter().any(|t| t == "Judul"));
        assert!(run_texts(st, &fmt(Run { bold: true, ..PLAIN })).iter().any(|t| t == "tebal"));
        // Any italic run is the `_miring_` emphasis or the `>` quote.
        let mut italic_texts = Vec::new();
        let mut at = 0usize;
        for r in &st.chars {
            if r.format.over.font_style.as_deref() == Some("Italic")
                && let Some(t) = st.text.get(at..at + r.len)
            {
                italic_texts.push(t.trim_end_matches('\n').to_string());
            }
            at += r.len;
        }
        assert!(italic_texts.iter().any(|t| t == "miring"), "{italic_texts:?}");
        assert!(italic_texts.iter().any(|t| t == "kutipan"), "{italic_texts:?}");
        assert!(run_texts(st, &fmt(PLAIN)).iter().any(|t| t.contains("kode")));
    }

    #[test]
    fn literal_asterisks_and_soft_wrap() {
        let i = import("2 * 3 = 6\nsoft wrapped\nline\n\nhard  \nbreak");
        assert_eq!(i.story.text, format!("2 * 3 = 6 soft wrapped line\nhard{}break", designcraft_doc::FORCED_LINE_BREAK));
    }

    #[test]
    fn fenced_code_keeps_lines() {
        let i = import("```\nsatu\ndua\n```");
        assert_eq!(i.story.text, "satu\ndua");
        assert_eq!(i.story.paras.len(), 2);
    }
}
