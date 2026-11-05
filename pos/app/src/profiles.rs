#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PubgenProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sector: ProfileSector,
    pub geometry: ProfileGeometry,
    pub structure: ProfileStructure,
    pub nomenclature: ProfileNomenclature,
    pub sheet_routing: ProfileSheetRouting,
    pub table_styling: ProfileTableStyling,
    #[serde(default)]
    pub typography: ProfileTypography,
}

impl Default for PubgenProfile {
    fn default() -> Self {
        ProfileManager::default_profile()
    }
}

impl PubgenProfile {
    pub fn sanitize(&mut self) {
        self.geometry.margins_mm.sanitize();
        if self.id.trim().is_empty() {
            self.id = "custom".to_string();
        }
        if self.name.trim().is_empty() {
            self.name = "Custom Profile".to_string();
        }
    }

    /// Metadata form fields shown for this profile (dynamic per publication profile).
    pub fn metadata_fields(&self) -> Vec<crate::metadata_form::MetaField> {
        use crate::metadata_form::MetaField as F;
        match self.id.as_str() {
            "bps_brs" => vec![F::ReleaseNo, F::ReleaseDate, F::ReleaseMonth, F::Title],
            "academic_monograph" => vec![F::Title, F::Subtitle, F::Authors, F::Year, F::Code],
            "corporate_annual" => vec![F::Title, F::Subtitle, F::Publisher, F::Year, F::Catalogue],
            _ => vec![F::Title, F::TitleEn, F::Subtitle, F::Year, F::Volume, F::Catalogue, F::Code],
        }
    }
}

impl std::fmt::Display for PubgenProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProfileSector {
    #[default]
    BpsOfficial,
    Government,
    Corporate,
    Academic,
    Custom,
}

impl ProfileSector {
    pub fn label(self) -> &'static str {
        match self {
            Self::BpsOfficial => "BPS Official",
            Self::Government => "Pemerintahan / LPPD",
            Self::Corporate => "Korporat & Bisnis",
            Self::Academic => "Akademik & Riset",
            Self::Custom => "Kustom",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileGeometry {
    pub page_size: String,
    pub facing_pages: bool,
    pub margins_mm: MarginsMm,
}

impl Default for ProfileGeometry {
    fn default() -> Self {
        Self {
            page_size: "A4".to_string(),
            facing_pages: true,
            margins_mm: MarginsMm::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarginsMm {
    pub top: f64,
    pub bottom: f64,
    pub inside: f64,
    pub outside: f64,
}

impl Default for MarginsMm {
    fn default() -> Self {
        Self {
            top: 20.2,
            bottom: 20.0,
            inside: 19.0,
            outside: 15.0,
        }
    }
}

impl MarginsMm {
    pub fn sanitize(&mut self) {
        self.top = self.top.clamp(5.0, 100.0);
        self.bottom = self.bottom.clamp(5.0, 100.0);
        self.inside = self.inside.clamp(5.0, 100.0);
        self.outside = self.outside.clamp(5.0, 100.0);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileStructure {
    pub has_front_matter: bool,
    pub front_matter_numbering: String, // "roman_lower", "arabic", "none"
    pub restart_body_numbering: bool,
    pub require_chapter_dividers: bool,
    pub include_inside_cover: bool,
    pub include_back_cover: bool,
    #[serde(default = "default_true")]
    pub bilingual: bool,
    #[serde(default)]
    pub header_footer_preset: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for ProfileStructure {
    fn default() -> Self {
        Self {
            has_front_matter: true,
            front_matter_numbering: "roman_lower".to_string(),
            restart_body_numbering: true,
            require_chapter_dividers: true,
            include_inside_cover: true,
            include_back_cover: true,
            bilingual: true,
            header_footer_preset: Some("hf-dda".to_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileNomenclature {
    pub chapter_prefix: String,
    pub table_prefix: String,
    pub chart_prefix: String,
    pub toc_title: String,
    pub list_tables_title: String,
    pub list_charts_title: String,
    pub source_labels: Vec<String>,
    pub note_labels: Vec<String>,
}

impl Default for ProfileNomenclature {
    fn default() -> Self {
        Self {
            chapter_prefix: "BAB".to_string(),
            table_prefix: "Tabel".to_string(),
            chart_prefix: "Grafik".to_string(),
            toc_title: "Daftar Isi".to_string(),
            list_tables_title: "Daftar Tabel".to_string(),
            list_charts_title: "Daftar Grafik".to_string(),
            source_labels: vec!["Sumber:".to_string(), "Source:".to_string(), "Sumber / Source:".to_string()],
            note_labels: vec!["Catatan:".to_string(), "Note:".to_string(), "Catatan / Note:".to_string(), "Keterangan:".to_string()],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ProfileSheetRouting {
    pub cover_patterns: Vec<String>,
    pub divider_patterns: Vec<String>,
    pub narrative_patterns: Vec<String>,
    pub table_patterns: Vec<String>,
    pub chart_patterns: Vec<String>,
    #[serde(default)]
    pub graphic_patterns: Vec<String>,
}

fn matches_pattern(pattern: &str, s: &str) -> bool {
    let s_clean = s.trim().to_lowercase();
    let pat = pattern.trim().to_lowercase();
    let pat_clean = pat.strip_prefix("(?i)").unwrap_or(&pat);
    let pat_clean = pat_clean.trim_start_matches('^').trim_end_matches('$');

    if pat_clean.ends_with(".*") {
        let prefix = pat_clean.trim_end_matches(".*");
        s_clean.starts_with(prefix)
    } else if pat_clean.contains(".*") {
        let parts: Vec<&str> = pat_clean.split(".*").collect();
        if parts.len() == 2 {
            s_clean.starts_with(parts[0]) && s_clean.ends_with(parts[1])
        } else {
            s_clean.contains(pat_clean)
        }
    } else {
        s_clean == pat_clean || s_clean.starts_with(pat_clean)
    }
}

impl ProfileSheetRouting {
    pub fn matches_cover(&self, sheet_name: &str) -> bool {
        self.cover_patterns.iter().any(|p| matches_pattern(p, sheet_name))
    }

    pub fn matches_divider(&self, sheet_name: &str) -> bool {
        self.divider_patterns.iter().any(|p| matches_pattern(p, sheet_name))
    }

    pub fn matches_narrative(&self, sheet_name: &str) -> bool {
        self.narrative_patterns.iter().any(|p| matches_pattern(p, sheet_name))
    }

    pub fn matches_table(&self, sheet_name: &str) -> bool {
        self.table_patterns.iter().any(|p| matches_pattern(p, sheet_name))
    }

    pub fn matches_chart(&self, sheet_name: &str) -> bool {
        self.chart_patterns.iter().any(|p| matches_pattern(p, sheet_name))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileTableStyling {
    pub include_column_number_row: bool,
    #[serde(default = "default_true")]
    pub include_table_footer: bool,
    pub alternating_row_fills: bool,
    pub horizontal_rules_only: bool,
    pub default_theme_palette: String,
}

impl Default for ProfileTableStyling {
    fn default() -> Self {
        Self {
            include_column_number_row: true,
            include_table_footer: true,
            alternating_row_fills: true,
            horizontal_rules_only: false,
            default_theme_palette: "bps_teal".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileTypography {
    pub body_font: String,
    pub heading_font: String,
    pub title_font: String,
}

impl Default for ProfileTypography {
    fn default() -> Self {
        Self {
            body_font: "Arial".to_string(),
            heading_font: "Arial".to_string(),
            title_font: "Arial Black".to_string(),
        }
    }
}

pub struct ProfileManager;

/// Built-in profiles, loaded from the JSON definitions in `assets/profiles/`
/// (shared with the DesignCraft reference app).
pub const PROFILE_JSONS: [(&str, &str); 4] = [
    ("bps_dda", include_str!("../assets/profiles/bps_dda.json")),
    ("bps_brs", include_str!("../assets/profiles/bps_brs.json")),
    ("academic_monograph", include_str!("../assets/profiles/academic_monograph.json")),
    ("corporate_annual", include_str!("../assets/profiles/corporate_annual.json")),
];

pub fn loaded_profiles() -> &'static [PubgenProfile] {
    static PROFILES: std::sync::OnceLock<Vec<PubgenProfile>> = std::sync::OnceLock::new();
    PROFILES.get_or_init(|| {
        PROFILE_JSONS
            .iter()
            .map(|(id, json)| {
                if let Ok(meta) = serde_json::from_str::<crate::metadata_form::Metadata>(json) {
                    PubgenProfile::from_metadata(&meta)
                } else if let Ok(p) = serde_json::from_str::<PubgenProfile>(json) {
                    p
                } else {
                    ProfileManager::fallback_by_id(id)
                }
            })
            .collect()
    })
}

impl PubgenProfile {
    pub fn from_metadata(meta: &crate::metadata_form::Metadata) -> Self {
        let is_brs = meta.profile_id.trim().to_lowercase().contains("brs");
        let is_corporate = meta.profile_id.trim().to_lowercase().contains("corporate");
        let is_academic = meta.profile_id.trim().to_lowercase().contains("academic");

        let mut prof = if is_brs {
            ProfileManager::bps_brs()
        } else if is_corporate {
            ProfileManager::corporate_annual()
        } else if is_academic {
            ProfileManager::academic_monograph()
        } else {
            ProfileManager::bps_dda()
        };

        prof.id = meta.profile_id.clone();
        if !meta.name.trim().is_empty() {
            prof.name = meta.name.clone();
        }
        if !meta.description.trim().is_empty() {
            prof.description = meta.description.clone();
        }
        prof.geometry.facing_pages = meta.layout == crate::metadata_form::PageLayout::Double;
        prof.geometry.margins_mm = MarginsMm {
            top: meta.margins.top as f64,
            bottom: meta.margins.bottom as f64,
            inside: meta.margins.left as f64,
            outside: meta.margins.right as f64,
        };
        prof.structure.bilingual = meta.lang == crate::metadata_form::Lang::IdEn;
        prof
    }
}

impl ProfileManager {
    pub fn default_profile() -> PubgenProfile {
        Self::get_profile("bps_dda")
    }

    pub fn builtin_profiles() -> Vec<PubgenProfile> {
        let loaded = loaded_profiles();
        if loaded.is_empty() {
            vec![Self::bps_dda(), Self::bps_brs(), Self::academic_monograph(), Self::corporate_annual()]
        } else {
            loaded.to_vec()
        }
    }

    pub fn get_profile(id: &str) -> PubgenProfile {
        let clean = id.trim().to_lowercase();
        if let Some(hit) = loaded_profiles().iter().find(|p| p.id == clean) {
            return hit.clone();
        }
        Self::fallback_by_id(&clean)
    }

    pub fn fallback_by_id(clean: &str) -> PubgenProfile {
        match clean {
            "bps_brs" | "brs" => Self::bps_brs(),
            "academic_monograph" => Self::academic_monograph(),
            "corporate_annual" => Self::corporate_annual(),
            _ => Self::bps_dda(),
        }
    }

    pub fn bps_dda() -> PubgenProfile {
        PubgenProfile {
            id: "bps_dda".to_string(),
            name: "BPS Daerah Dalam Angka".to_string(),
            description: "Publikasi standar BPS Kabupaten/Kota/Kecamatan Dalam Angka dengan nomor kolom tabel dan pembatas bab.".to_string(),
            sector: ProfileSector::BpsOfficial,
            geometry: ProfileGeometry {
                page_size: "A4".to_string(),
                facing_pages: true,
                margins_mm: MarginsMm { top: 20.2, bottom: 20.0, inside: 19.0, outside: 15.0 },
            },
            structure: ProfileStructure {
                has_front_matter: true,
                front_matter_numbering: "roman_lower".to_string(),
                restart_body_numbering: true,
                require_chapter_dividers: true,
                include_inside_cover: true,
                include_back_cover: true,
                bilingual: true,
                header_footer_preset: Some("hf-dda".to_string()),
            },
            nomenclature: ProfileNomenclature::default(),
            sheet_routing: ProfileSheetRouting {
                cover_patterns: vec!["cover".to_string(), "sampul".to_string()],
                divider_patterns: vec!["div.*".to_string(), "pembatas.*".to_string()],
                narrative_patterns: vec!["r\\d+.*".to_string(), "ulasan.*".to_string()],
                table_patterns: vec!["\\d+[_.]\\d+.*".to_string(), "tabel.*".to_string(), "table.*".to_string()],
                chart_patterns: vec!["c\\d+.*".to_string(), "chart.*".to_string(), "grafik.*".to_string()],
                graphic_patterns: vec!["g\\d+.*".to_string(), "gambar.*".to_string()],
            },
            table_styling: ProfileTableStyling {
                include_column_number_row: true,
                include_table_footer: true,
                alternating_row_fills: true,
                horizontal_rules_only: false,
                default_theme_palette: "bps_teal".to_string(),
            },
            typography: ProfileTypography::default(),
        }
    }

    pub fn bps_brs() -> PubgenProfile {
        PubgenProfile {
            id: "bps_brs".to_string(),
            name: "BPS Berita Resmi Statistik".to_string(),
            description: "Press release dan rilis statistik berkala BPS, format halaman tunggal modern tanpa nomor kolom tabel.".to_string(),
            sector: ProfileSector::BpsOfficial,
            geometry: ProfileGeometry {
                page_size: "A4".to_string(),
                facing_pages: false,
                margins_mm: MarginsMm { top: 20.2, bottom: 20.0, inside: 19.0, outside: 15.0 },
            },
            structure: ProfileStructure {
                has_front_matter: false,
                front_matter_numbering: "none".to_string(),
                restart_body_numbering: false,
                require_chapter_dividers: false,
                include_inside_cover: false,
                include_back_cover: true,
                bilingual: false,
                header_footer_preset: Some("hf-brs".to_string()),
            },
            nomenclature: ProfileNomenclature {
                chapter_prefix: String::new(),
                table_prefix: "Tabel".to_string(),
                chart_prefix: "Grafik".to_string(),
                toc_title: "Daftar Isi".to_string(),
                list_tables_title: "Daftar Tabel".to_string(),
                list_charts_title: "Daftar Grafik".to_string(),
                source_labels: vec!["Sumber:".to_string(), "Source:".to_string(), "Data:".to_string()],
                note_labels: vec!["Catatan:".to_string(), "Note:".to_string(), "Notes:".to_string()],
            },
            sheet_routing: ProfileSheetRouting {
                cover_patterns: vec!["cov".to_string(), "cover".to_string(), "sampul".to_string()],
                divider_patterns: vec!["ringkasan.*".to_string(), "section.*".to_string()],
                narrative_patterns: vec!["point".to_string(), "u\\d+.*".to_string(), "brs.*".to_string(), "utama.*".to_string(), "ringkasan.*".to_string(), "ulasan.*".to_string()],
                table_patterns: vec!["t\\d+.*".to_string(), "tabel.*".to_string(), "table.*".to_string(), "\\d+[_.]\\d+.*".to_string()],
                chart_patterns: vec!["c\\d+.*".to_string(), "grafik.*".to_string(), "chart.*".to_string()],
                graphic_patterns: vec!["g\\d+.*".to_string(), "gambar.*".to_string(), "infografik.*".to_string()],
            },
            table_styling: ProfileTableStyling {
                include_column_number_row: false,
                include_table_footer: false,
                alternating_row_fills: true,
                horizontal_rules_only: false,
                default_theme_palette: "BRS6".to_string(),
            },
            typography: ProfileTypography {
                body_font: "Lato".to_string(),
                heading_font: "Lato".to_string(),
                title_font: "Lato".to_string(),
            },
        }
    }

    pub fn academic_monograph() -> PubgenProfile {
        PubgenProfile {
            id: "academic_monograph".to_string(),
            name: "Academic & Think Tank Monograph".to_string(),
            description: "Monograf akademik, laporan penelitian, dan whitepaper kebijakan dengan margin asimetris untuk anotasi.".to_string(),
            sector: ProfileSector::Academic,
            geometry: ProfileGeometry {
                page_size: "A4".to_string(),
                facing_pages: true,
                margins_mm: MarginsMm { top: 25.0, bottom: 25.0, inside: 20.0, outside: 25.0 },
            },
            structure: ProfileStructure {
                has_front_matter: true,
                front_matter_numbering: "roman_lower".to_string(),
                restart_body_numbering: true,
                require_chapter_dividers: false,
                include_inside_cover: false,
                include_back_cover: true,
                bilingual: false,
                header_footer_preset: Some("hf-minimal".to_string()),
            },
            nomenclature: ProfileNomenclature {
                chapter_prefix: "Section".to_string(),
                table_prefix: "Table".to_string(),
                chart_prefix: "Figure".to_string(),
                toc_title: "Table of Contents".to_string(),
                list_tables_title: "List of Tables".to_string(),
                list_charts_title: "List of Figures".to_string(),
                source_labels: vec!["Source:".to_string(), "Data:".to_string()],
                note_labels: vec!["Note:".to_string(), "Notes:".to_string()],
            },
            sheet_routing: ProfileSheetRouting {
                cover_patterns: vec!["cover".to_string()],
                divider_patterns: vec!["section.*".to_string()],
                narrative_patterns: vec!["intro.*".to_string(), "chapter.*".to_string(), "text.*".to_string()],
                table_patterns: vec!["table.*".to_string(), "t\\d+.*".to_string()],
                chart_patterns: vec!["fig.*".to_string(), "figure.*".to_string(), "chart.*".to_string()],
                graphic_patterns: vec!["img.*".to_string(), "image.*".to_string()],
            },
            table_styling: ProfileTableStyling {
                include_column_number_row: false,
                include_table_footer: true,
                alternating_row_fills: false,
                horizontal_rules_only: true,
                default_theme_palette: "monochrome".to_string(),
            },
            typography: ProfileTypography {
                body_font: "Times New Roman".to_string(),
                heading_font: "Arial".to_string(),
                title_font: "Arial Black".to_string(),
            },
        }
    }

    pub fn corporate_annual() -> PubgenProfile {
        PubgenProfile {
            id: "corporate_annual".to_string(),
            name: "Corporate Annual Report".to_string(),
            description: "Laporan tahunan perusahaan publik dan BUMN dengan tata letak visual modern dan tabel keuangan elegan.".to_string(),
            sector: ProfileSector::Corporate,
            geometry: ProfileGeometry {
                page_size: "A4".to_string(),
                facing_pages: true,
                margins_mm: MarginsMm { top: 20.0, bottom: 20.0, inside: 20.0, outside: 20.0 },
            },
            structure: ProfileStructure {
                has_front_matter: true,
                front_matter_numbering: "roman_lower".to_string(),
                restart_body_numbering: true,
                require_chapter_dividers: true,
                include_inside_cover: true,
                include_back_cover: true,
                bilingual: true,
                header_footer_preset: Some("hf-corporate".to_string()),
            },
            nomenclature: ProfileNomenclature {
                chapter_prefix: "BAB".to_string(),
                table_prefix: "Tabel".to_string(),
                chart_prefix: "Grafik".to_string(),
                toc_title: "Daftar Isi".to_string(),
                list_tables_title: "Daftar Tabel".to_string(),
                list_charts_title: "Daftar Grafik".to_string(),
                source_labels: vec!["Sumber:".to_string(), "Source:".to_string()],
                note_labels: vec!["Catatan:".to_string(), "Note:".to_string()],
            },
            sheet_routing: ProfileSheetRouting {
                cover_patterns: vec!["cover".to_string(), "sampul".to_string()],
                divider_patterns: vec!["div.*".to_string()],
                narrative_patterns: vec!["laporan.*".to_string(), "kinerja.*".to_string()],
                table_patterns: vec!["keuangan.*".to_string(), "tabel.*".to_string(), "table.*".to_string()],
                chart_patterns: vec!["grafik.*".to_string(), "chart.*".to_string()],
                graphic_patterns: vec!["foto.*".to_string(), "image.*".to_string()],
            },
            table_styling: ProfileTableStyling {
                include_column_number_row: false,
                include_table_footer: true,
                alternating_row_fills: true,
                horizontal_rules_only: false,
                default_theme_palette: "corporate_navy".to_string(),
            },
            typography: ProfileTypography {
                body_font: "Segoe UI".to_string(),
                heading_font: "Segoe UI".to_string(),
                title_font: "Segoe UI Bold".to_string(),
            },
        }
    }
}

/// Auto-detects the most suitable publication profile ID based on sheet names in the workbook.
pub fn detect_profile_from_sheets(sheet_names: &[impl AsRef<str>]) -> &'static str {
    for name in sheet_names {
        let s = name.as_ref().trim();
        let lower = s.to_lowercase();
        if lower.starts_with("brs")
            || lower.contains("ringkasan")
            || lower.contains("utama")
            || lower == "point"
            || (lower == "cov"
                && sheet_names.iter().any(|n| {
                    let nl = n.as_ref().trim().to_lowercase();
                    nl == "point" || nl.starts_with('u') || nl.starts_with('t')
                }))
        {
            return "bps_brs";
        }
    }
    "bps_dda"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_profiles() {
        // All four JSON-defined profiles load from assets/profiles/.
        let builtins = ProfileManager::builtin_profiles();
        assert_eq!(builtins.len(), 4, "ids: {:?}", builtins.iter().map(|p| p.id.as_str()).collect::<Vec<_>>());
        assert!(builtins.iter().any(|p| p.id == "academic_monograph"));
        assert!(builtins.iter().any(|p| p.id == "corporate_annual"));

        let dda = ProfileManager::get_profile("bps_dda");
        assert_eq!(dda.id, "bps_dda");
        assert!(dda.geometry.facing_pages);
        assert_eq!(dda.geometry.margins_mm.top, 20.2);

        let brs = ProfileManager::get_profile("bps_brs");
        assert_eq!(brs.id, "bps_brs");
        assert_eq!(brs.geometry.margins_mm.top, 20.2);

        // Metadata fields are dynamic per profile.
        assert!(dda.metadata_fields().contains(&crate::metadata_form::MetaField::TitleEn));
        assert!(!brs.metadata_fields().contains(&crate::metadata_form::MetaField::TitleEn));
        assert!(brs.metadata_fields().contains(&crate::metadata_form::MetaField::ReleaseNo));
        assert!(ProfileManager::get_profile("academic_monograph")
            .metadata_fields()
            .contains(&crate::metadata_form::MetaField::Authors));
        assert!(ProfileManager::get_profile("corporate_annual")
            .metadata_fields()
            .contains(&crate::metadata_form::MetaField::Publisher));
    }

    #[test]
    fn test_detect_profile() {
        assert_eq!(detect_profile_from_sheets(&["COVER", "R01_Geografi", "1.1_Luas"]), "bps_dda");
        assert_eq!(detect_profile_from_sheets(&["COV", "BRS_Pertumbuhan", "T1_Data"]), "bps_brs");
        assert_eq!(detect_profile_from_sheets(&["COVER", "Point", "Ringkasan"]), "bps_brs");
    }
}
