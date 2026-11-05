//! Single canonical source for publication profiles, margins, and geometry.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};

/// Predefined canonical publication profile identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProfileId {
    BpsBrs,
    BpsDda,
    GenericBook,
    GenericReport,
}

impl ProfileId {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "bps_brs" | "brs" => Some(Self::BpsBrs),
            "bps_dda" | "dda" => Some(Self::BpsDda),
            "generic_book" | "book" => Some(Self::GenericBook),
            "generic_report" | "report" => Some(Self::GenericReport),
            _ => None,
        }
    }

    pub fn id_str(&self) -> &'static str {
        match self {
            Self::BpsBrs => "bps_brs",
            Self::BpsDda => "bps_dda",
            Self::GenericBook => "generic_book",
            Self::GenericReport => "generic_report",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::BpsBrs => "BPS Berita Resmi Statistik (BRS)",
            Self::BpsDda => "BPS Daerah Dalam Angka (DDA)",
            Self::GenericBook => "Buku Standar / General Book",
            Self::GenericReport => "Laporan Publikasi / General Report",
        }
    }
}

/// Page geometry and margin specifications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutGeometry {
    pub paper_size: String,
    pub width_mm: f32,
    pub height_mm: f32,
    pub margin_top_mm: f32,
    pub margin_bottom_mm: f32,
    pub margin_left_mm: f32,
    pub margin_right_mm: f32,
    pub facing_pages: bool,
}

impl LayoutGeometry {
    pub fn width_pt(&self) -> f32 {
        self.width_mm * 72.0 / 25.4
    }

    pub fn height_pt(&self) -> f32 {
        self.height_mm * 72.0 / 25.4
    }
}

/// Canonical publication profile definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicationProfile {
    pub id: ProfileId,
    pub geometry: LayoutGeometry,
    pub bilingual: bool,
    pub running_header_template: String,
    pub running_footer_template: String,
    pub table_alternating_rows: bool,
    pub toc_show_leaders: bool,
}

/// Profile service providing canonical defaults.
pub struct ProfileService;

impl ProfileService {
    pub fn resolve(id_str: &str) -> Option<PublicationProfile> {
        let pid = ProfileId::parse(id_str)?;
        Some(Self::for_id(pid))
    }

    pub fn for_id(id: ProfileId) -> PublicationProfile {
        match id {
            ProfileId::BpsBrs => PublicationProfile {
                id: ProfileId::BpsBrs,
                geometry: LayoutGeometry {
                    paper_size: "A4".to_string(),
                    width_mm: 210.0,
                    height_mm: 297.0,
                    margin_top_mm: 18.0,
                    margin_bottom_mm: 18.0,
                    margin_left_mm: 20.0,
                    margin_right_mm: 20.0,
                    facing_pages: true,
                },
                bilingual: false,
                running_header_template: "BERITA RESMI STATISTIK".to_string(),
                running_footer_template: "Halaman {page}".to_string(),
                table_alternating_rows: false,
                toc_show_leaders: true,
            },
            ProfileId::BpsDda => PublicationProfile {
                id: ProfileId::BpsDda,
                geometry: LayoutGeometry {
                    paper_size: "B5".to_string(),
                    width_mm: 176.0,
                    height_mm: 250.0,
                    margin_top_mm: 20.0,
                    margin_bottom_mm: 20.0,
                    margin_left_mm: 25.0,
                    margin_right_mm: 20.0,
                    facing_pages: true,
                },
                bilingual: true,
                running_header_template: "PROVINSI JAWA BARAT DALAM ANGKA".to_string(),
                running_footer_template: "{page} | BPS PROVINSI JAWA BARAT".to_string(),
                table_alternating_rows: true,
                toc_show_leaders: true,
            },
            ProfileId::GenericBook => PublicationProfile {
                id: ProfileId::GenericBook,
                geometry: LayoutGeometry {
                    paper_size: "A5".to_string(),
                    width_mm: 148.0,
                    height_mm: 210.0,
                    margin_top_mm: 15.0,
                    margin_bottom_mm: 15.0,
                    margin_left_mm: 20.0,
                    margin_right_mm: 15.0,
                    facing_pages: true,
                },
                bilingual: false,
                running_header_template: "{chapter_title}".to_string(),
                running_footer_template: "{page}".to_string(),
                table_alternating_rows: false,
                toc_show_leaders: true,
            },
            ProfileId::GenericReport => PublicationProfile {
                id: ProfileId::GenericReport,
                geometry: LayoutGeometry {
                    paper_size: "A4".to_string(),
                    width_mm: 210.0,
                    height_mm: 297.0,
                    margin_top_mm: 20.0,
                    margin_bottom_mm: 20.0,
                    margin_left_mm: 20.0,
                    margin_right_mm: 20.0,
                    facing_pages: false,
                },
                bilingual: false,
                running_header_template: "{title}".to_string(),
                running_footer_template: "{page}".to_string(),
                table_alternating_rows: true,
                toc_show_leaders: false,
            },
        }
    }
}
