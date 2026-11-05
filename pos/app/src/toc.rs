use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::metadata_form::{PageEntry, PageKind, TocLayout, TocStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TocKind {
    Contents, // Daftar Isi (Bab & Ulasan)
    Tables,   // Daftar Tabel
    Charts,   // Daftar Grafik
}

impl TocKind {
    pub fn title(self, bilingual: bool) -> &'static str {
        match (self, bilingual) {
            (Self::Contents, false) => "DAFTAR ISI",
            (Self::Contents, true) => "DAFTAR ISI / TABLE OF CONTENTS",
            (Self::Tables, false) => "DAFTAR TABEL",
            (Self::Tables, true) => "DAFTAR TABEL / LIST OF TABLES",
            (Self::Charts, false) => "DAFTAR GRAFIK",
            (Self::Charts, true) => "DAFTAR GRAFIK / LIST OF CHARTS",
        }
    }

    pub fn matches_prefix(self, line: &str) -> bool {
        let trimmed = line.trim().to_lowercase();
        match self {
            Self::Contents => {
                trimmed.starts_with("bab") || trimmed.starts_with("chapter")
            }
            Self::Tables => {
                trimmed.starts_with("tabel") || trimmed.starts_with("table")
            }
            Self::Charts => {
                trimmed.starts_with("grafik")
                    || trimmed.starts_with("chart")
                    || trimmed.starts_with("gambar")
                    || trimmed.starts_with("figure")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TocEntry {
    pub kind: TocKind,
    pub title: String,
    pub page: usize,
    pub level: usize,
    pub bilingual_title: Option<String>,
}

/// Extracts TOC entries from DesignCraft document inspect JSON structure.
pub fn extract_toc_from_inspect(doc: &Value, kind: TocKind) -> Vec<TocEntry> {
    let mut entries = Vec::new();

    // Map story ID -> page number (first frame placed on a page)
    let mut story_to_page = std::collections::HashMap::new();
    if let Some(spreads) = doc.get("spreads").and_then(Value::as_array) {
        for (page_idx, spread) in spreads.iter().enumerate() {
            if let Some(items) = spread.get("items").and_then(Value::as_array) {
                for item in items {
                    if let Some(tf) = item.get("text_frame")
                        && let Some(sid) = tf.get("story").and_then(Value::as_u64)
                    {
                        story_to_page.entry(sid).or_insert(page_idx + 1);
                    }
                }
            }
        }
    }

    // Traverse stories to find matching headers/titles
    if let Some(stories) = doc.get("stories").and_then(Value::as_array) {
        for story in stories {
            let sid = story.get("id").and_then(Value::as_u64).unwrap_or(0);
            let page = story_to_page.get(&sid).copied().unwrap_or(1);
            let text = story.get("text").and_then(Value::as_str).unwrap_or("");

            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if kind.matches_prefix(trimmed) {
                    let level = if kind == TocKind::Contents && trimmed.to_lowercase().starts_with("bab") {
                        1
                    } else {
                        2
                    };
                    entries.push(TocEntry {
                        kind,
                        title: trimmed.to_string(),
                        page,
                        level,
                        bilingual_title: None,
                    });
                }
            }
        }
    }

    entries
}

/// Generates formatted text ready to be inserted into a TOC text frame.
/// Formatted with right-indent tab (`\t` or `\u{e006}`) and page numbers.
pub fn format_toc_story_text(entries: &[TocEntry], bilingual: bool, kind: TocKind) -> String {
    let mut out = String::new();
    out.push_str(kind.title(bilingual));
    out.push_str("\n\n");

    for entry in entries {
        let indent = if entry.level > 1 { "   " } else { "" };
        let title_line = if let Some(ref bi) = entry.bilingual_title {
            if bilingual {
                format!("{} / {}", entry.title, bi)
            } else {
                entry.title.clone()
            }
        } else {
            entry.title.clone()
        };

        out.push_str(indent);
        out.push_str(&title_line);
        out.push('\t');
        out.push_str(&entry.page.to_string());
        out.push('\n');
    }

    out
}

// ---- TOC cards (Page Structure kind `PageKind::Toc`) ----------------------
//
// Mirrors the reference pubgen TOC engine (`TocPlan`, `toc_kind_words`,
// `toc_header_line`): the batch script talks to DesignCraft's `toc.generate`
// command with the same entry styles, `pageTitle` filters and `autoflow` rules.

/// The narrative page style of a publication, per the reference DDA template.
pub const TOC_NARRATIVE_STYLE: &str = "TOC Chapter";
/// The plain tagged paragraph style TOC tables/charts/entries list.
pub const TOC_ENTRY_STYLE: &str = "TOC Entry";

/// The paragraph of a TOC row family: the reference prefixes entry styles so that
/// regenerating one list never restyles another's pages.
fn entry_style_prefix(style: TocStyle) -> &'static str {
    match style {
        TocStyle::Table => "TOC Table Level",
        TocStyle::Chart => "TOC Chart Level",
        _ => "TOC Level",
    }
}

/// The words a TOC style lists pages by, matching the page's first text box
/// (reference `toc_kind_words`). `Entry` lists every tagged paragraph.
pub fn toc_style_words(style: TocStyle) -> &'static [&'static str] {
    match style {
        TocStyle::Table => &["Tabel", "Table"],
        TocStyle::Chart => &["Grafik", "Chart"],
        TocStyle::Paragraph => &["BAB"],
        TocStyle::Entry => &[],
    }
}

/// The `toc.generate` `entries` array a TOC card command carries — the same
/// levels, filters and entry styles the reference `TocPlan::entries_json` emits.
pub fn toc_entries_json(style: TocStyle, pair_bilingual: bool) -> Value {
    let entry = |style_name: &str, level: u8, words: &[&str], chapter_number: bool| {
        let mut e = serde_json::Map::new();
        e.insert("style".into(), json!(style_name));
        e.insert("level".into(), json!(level));
        e.insert("entryStyle".into(), json!(format!("{} {level}", entry_style_prefix(style))));
        e.insert("pageNumber".into(), json!("after"));
        e.insert("separator".into(), json!("\t"));
        if !words.is_empty() {
            e.insert("pageTitle".into(), json!(words));
        }
        if chapter_number {
            e.insert("chapterNumber".into(), json!(true));
        }
        if pair_bilingual {
            e.insert("pairColumns".into(), json!(true));
        }
        Value::Object(e)
    };
    let levels: Vec<Value> = match style {
        TocStyle::Paragraph => vec![
            entry(TOC_NARRATIVE_STYLE, 1, &[], true),
            entry("Heading 1", 2, &[], false),
            entry("Heading 2", 3, &[], false),
        ],
        TocStyle::Entry => vec![
            entry(TOC_ENTRY_STYLE, 1, &[], false),
            entry(TOC_NARRATIVE_STYLE, 1, &[], false),
        ],
        _ => vec![entry(TOC_ENTRY_STYLE, 1, toc_style_words(style), false)],
    };
    Value::Array(levels)
}

/// The first column header a TablePage TOC writes (reference `toc_header_line`).
pub fn toc_header_line(style: TocStyle, bilingual: bool) -> String {
    // Same word pairs the `pageTitle` filters use, so header and filter never drift.
    let (id_type, en_type) = match toc_style_words(style) {
        [a, b, ..] => (*a, *b),
        _ => ("Tabel", "Table"),
    };
    if bilingual {
        format!("{id_type}\tHalaman\u{2028}{en_type}\tPage")
    } else {
        format!("{id_type}\thalaman")
    }
}

/// Structural pagination plan for the batch script: every `NewPage` item takes
/// one absolute page, `NewObject` flows on the current one, a cover takes
/// `cover_pages` pages (a facing spread covers two). A TOC card takes its
/// *estimated* final page span — and the script reserves exactly that many
/// pages via `layout.pages.insert {"count": span}` — so later cards keep
/// correct targets; any residual overflow is grown by DesignCraft `autoflow`.
pub struct PagePlan {
    /// First absolute 1-based page each card occupies (`pages[i]`), in order.
    pub page_of: Vec<usize>,
}

/// The page span a TOC card reserves: shared by the pagination model and the
/// emitted `layout.pages.insert` count so the two never disagree.
pub fn toc_span_pages(pages: &[PageEntry], entry: &PageEntry) -> usize {
    let Some(t) = entry.toc.as_ref() else { return 1 };
    let entries = estimate_toc_entries(pages, t.style) + usize::from(t.include_self);
    estimate_toc_pages(entries, t.layout).max(1)
}

pub fn simulate_pagination(pages: &[PageEntry], cover_pages: usize) -> PagePlan {
    let mut page = 0usize;
    let mut page_of = Vec::with_capacity(pages.len());
    for entry in pages {
        match entry.kind {
            PageKind::Cover => {
                page += cover_pages.max(1);
                page_of.push(page - cover_pages.max(1) + 1);
            }
            PageKind::Toc => {
                let est = toc_span_pages(pages, entry);
                page += est;
                page_of.push(page - est + 1);
            }
            _ if entry.scope == crate::metadata_form::PageItemScope::NewObject => {
                if page == 0 {
                    page = 1;
                }
                page_of.push(page);
            }
            _ => {
                page += 1;
                page_of.push(page);
            }
        }
    }
    PagePlan { page_of }
}

/// Estimated entries per TOC page, per layout. Real overflow is handled by
/// DesignCraft's `toc.generate` autoflow; this only feeds placement estimates.
pub const TOC_ENTRIES_PER_PAGE: usize = 26;
pub const TOC_ENTRIES_PER_PAGE_TABLEPAGE: usize = 22;

pub fn estimate_toc_pages(entry_count: usize, layout: TocLayout) -> usize {
    let per = match layout {
        TocLayout::Standard => TOC_ENTRIES_PER_PAGE,
        TocLayout::TablePage => TOC_ENTRIES_PER_PAGE_TABLEPAGE,
    };
    entry_count.saturating_add(1).div_ceil(per.max(1)).max(1)
}

/// How many documentary lines the TOC of `style` lists from the page cards:
/// used only to estimate how many pages the generated TOC will need.
pub fn estimate_toc_entries(pages: &[PageEntry], style: TocStyle) -> usize {
    match style {
        TocStyle::Table => pages.iter().filter(|p| p.kind == PageKind::Table).count(),
        TocStyle::Chart => pages.iter().filter(|p| p.kind == PageKind::Chart).count(),
        TocStyle::Paragraph => pages.iter().filter(|p| p.kind == PageKind::Paragraph).count(),
        TocStyle::Entry => pages
            .iter()
            .filter(|p| matches!(p.kind, PageKind::Paragraph | PageKind::Table | PageKind::Chart))
            .count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_toc_kind_matching() {
        assert!(TocKind::Contents.matches_prefix("BAB 01 GEOGRAFI"));
        assert!(TocKind::Tables.matches_prefix("Tabel 1.1 Luas Wilayah"));
        assert!(TocKind::Charts.matches_prefix("Grafik 2.1 Trend Inflasi"));
        assert!(!TocKind::Contents.matches_prefix("Tabel 1.1"));
    }

    #[test]
    fn test_extract_toc_from_inspect() {
        let inspect_doc = json!({
            "spreads": [
                {
                    "items": [
                        { "text_frame": { "story": 10 } }
                    ]
                },
                {
                    "items": [
                        { "text_frame": { "story": 20 } }
                    ]
                }
            ],
            "stories": [
                {
                    "id": 10,
                    "text": "BAB 01 KEPENDUDUKAN\nUlasan singkat kependudukan..."
                },
                {
                    "id": 20,
                    "text": "Tabel 1.1 Jumlah Penduduk Menurut Kelompok Umur\n1 2 3"
                }
            ]
        });

        let bab_entries = extract_toc_from_inspect(&inspect_doc, TocKind::Contents);
        assert_eq!(bab_entries.len(), 1);
        assert_eq!(bab_entries[0].title, "BAB 01 KEPENDUDUKAN");
        assert_eq!(bab_entries[0].page, 1);

        let table_entries = extract_toc_from_inspect(&inspect_doc, TocKind::Tables);
        assert_eq!(table_entries.len(), 1);
        assert_eq!(table_entries[0].title, "Tabel 1.1 Jumlah Penduduk Menurut Kelompok Umur");
        assert_eq!(table_entries[0].page, 2);
    }

    #[test]
    fn test_format_toc_story_text() {
        let entries = vec![
            TocEntry {
                kind: TocKind::Contents,
                title: "BAB 01 GEOGRAFI".into(),
                page: 1,
                level: 1,
                bilingual_title: Some("Geography".into()),
            },
            TocEntry {
                kind: TocKind::Contents,
                title: "BAB 02 KEPENDUDUKAN".into(),
                page: 15,
                level: 1,
                bilingual_title: Some("Population".into()),
            },
        ];

        let formatted = format_toc_story_text(&entries, true, TocKind::Contents);
        assert!(formatted.contains("DAFTAR ISI / TABLE OF CONTENTS"));
        assert!(formatted.contains("BAB 01 GEOGRAFI / Geography\t1"));
        assert!(formatted.contains("BAB 02 KEPENDUDUKAN / Population\t15"));
    }

    #[test]
    fn test_toc_style_words_and_entries_json() {
        assert_eq!(toc_style_words(TocStyle::Table), &["Tabel", "Table"]);
        assert_eq!(toc_style_words(TocStyle::Chart), &["Grafik", "Chart"]);
        assert_eq!(toc_style_words(TocStyle::Paragraph), &["BAB"]);
        assert!(toc_style_words(TocStyle::Entry).is_empty());

        let tables = toc_entries_json(TocStyle::Table, false);
        let arr = tables.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["style"], "TOC Entry");
        assert_eq!(arr[0]["pageTitle"], json!(["Tabel", "Table"]));
        assert_eq!(arr[0]["entryStyle"], "TOC Table Level 1");

        let para = toc_entries_json(TocStyle::Paragraph, true);
        let arr = para.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["style"], "TOC Chapter");
        assert_eq!(arr[0]["chapterNumber"], true);
        assert_eq!(arr[0]["pairColumns"], true);
        assert_eq!(arr[1]["level"], 2);
        assert_eq!(arr[2]["entryStyle"], "TOC Level 3");
    }

    #[test]
    fn test_toc_header_line_bilingual() {
        assert_eq!(toc_header_line(TocStyle::Table, false), "Tabel\thalaman");
        assert_eq!(
            toc_header_line(TocStyle::Chart, true),
            "Grafik\tHalaman\u{2028}Chart\tPage"
        );
    }

    #[test]
    fn test_simulate_pagination_and_estimates() {
        use crate::metadata_form::{PageItemScope, PageKind, TocData, TocStyle as Ts};
        let pages = vec![
            PageEntry { kind: PageKind::Cover, scope: PageItemScope::NewPage, ..Default::default() },
            PageEntry { kind: PageKind::Toc, scope: PageItemScope::NewPage, toc: Some(TocData { style: Ts::Table, ..Default::default() }), ..Default::default() },
            PageEntry { kind: PageKind::Paragraph, scope: PageItemScope::NewPage, ..Default::default() },
            PageEntry { kind: PageKind::Table, scope: PageItemScope::NewPage, ..Default::default() },
            PageEntry { kind: PageKind::Table, scope: PageItemScope::NewObject, ..Default::default() },
            PageEntry { kind: PageKind::Table, scope: PageItemScope::NewPage, ..Default::default() },
        ];

        // Facing DDA: cover spread = pages 1-2, TOC on 3.
        let facing = simulate_pagination(&pages, 2);
        assert_eq!(facing.page_of, vec![1, 3, 4, 5, 5, 6]);
        assert_eq!(facing.page_of.last(), Some(&6));

        // Single sided: cover = 1 page, TOC on 2.
        let single = simulate_pagination(&pages, 1);
        assert_eq!(single.page_of, vec![1, 2, 3, 4, 4, 5]);
        assert_eq!(single.page_of.last(), Some(&5));

        assert_eq!(estimate_toc_pages(0, TocLayout::Standard), 1);
        assert_eq!(estimate_toc_pages(26, TocLayout::Standard), 2);
        assert_eq!(estimate_toc_entries(&pages, Ts::Table), 3);
    }
}

