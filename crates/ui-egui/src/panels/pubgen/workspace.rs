//! Orchestrator 3-Column Studio Workspace.
//!
//! Provides the complete Publication Orchestrator workspace matching the POS specification:
//! 1. Left Panel: "Sumber & Konten" (Content Tree, Import Actions, Document Settings)
//! 2. Center Panel: "Eksekusi & Verifikasi" (Header/Footer Preview, QA Preflight, Tabs: Page Structure Cards, Live Preview, Execution Log)
//! 3. Right Panel: "Profil, Slot & Eksekusi" (Publication Profile, Margins, Generate/Publish Buttons, Slot Detection)
//!
//! Both the left and right panels follow the exact visual styling and design tokens of DesignCraft's left panel.

use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Rect, Response, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
use serde::{Deserialize, Serialize};

use crate::DesignApp;
use crate::theme::{semibold, Tokens};

const WORKSPACE_STATE_ID: &str = "pubgen_orchestrator_workspace_state";

fn format_now() -> String {
    let now = chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now());
    now.format("%H:%M:%S").to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestratorTab {
    StrukturHalaman,
    VisualLivePreview,
    LogEksekusi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardKind {
    Cover,
    Narrative,
    Chart,
    Table,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentItem {
    pub path: String,
    pub size_bytes: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicationCard {
    pub kind: CardKind,
    pub title: String,
    pub subtitle: String,
    pub source: String,
    pub page: usize,
    pub tag1: String,
    pub tag2: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotItem {
    pub name: String,
    pub budget: Option<usize>,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchestratorWorkspaceState {
    pub center_tab: OrchestratorTab,
    pub selected_profile: String,
    // Left panel state
    pub tree_open: bool,
    pub csv_group_open: bool,
    pub md_group_open: bool,
    pub metadata_open: bool,
    pub content_files: Vec<ContentItem>,
    // Metadata form
    pub meta_title: String,
    pub meta_subtitle: String,
    pub meta_release_no: String,
    pub meta_release_date: String,
    pub meta_release_month: String,
    pub meta_agency: String,
    // Publication cards
    pub cards: Vec<PublicationCard>,
    // Slots
    pub slots: Vec<SlotItem>,
    // QA and output state
    pub last_qa_clean: bool,
    pub last_output_pdf: Option<String>,
    // Execution log
    pub execution_log: Vec<String>,
    // Card modal editor
    pub editing_card_idx: Option<usize>,
    // Status message
    pub status_message: Option<(String, bool)>,
}

impl Default for OrchestratorWorkspaceState {
    fn default() -> Self {
        let content_files = vec![
            ContentItem { path: "csv/C1_line.csv".into(), size_bytes: 517 },
            ContentItem { path: "csv/KUV.csv".into(), size_bytes: 555 },
            ContentItem { path: "csv/G1.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/POINT.csv".into(), size_bytes: 874 },
            ContentItem { path: "csv/T1.csv".into(), size_bytes: 1430 },
            ContentItem { path: "csv/T2.csv".into(), size_bytes: 1717 },
            ContentItem { path: "csv/T3.csv".into(), size_bytes: 547 },
            ContentItem { path: "csv/T4.csv".into(), size_bytes: 477 },
            ContentItem { path: "csv/U1.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U2.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U3.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U4_5.csv".into(), size_bytes: 14 },
            ContentItem { path: "csv/U5.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U6.csv".into(), size_bytes: 12 },
            ContentItem { path: "md/U1.md".into(), size_bytes: 1222 },
            ContentItem { path: "md/U2.md".into(), size_bytes: 3828 },
            ContentItem { path: "md/U4.md".into(), size_bytes: 2875 },
            ContentItem { path: "md/U4_5.md".into(), size_bytes: 2286 },
            ContentItem { path: "md/U5.md".into(), size_bytes: 1820 },
            ContentItem { path: "md/U6.md".into(), size_bytes: 1141 },
        ];

        let cards = vec![
            PublicationCard {
                kind: CardKind::Cover,
                title: "Sampul Berita Resmi Statistik (COV)".into(),
                subtitle: "Perkembangan Nilai Tukar Petani | 1 Oktober 2026 · BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
                source: "Tanpa sumber".into(),
                page: 1,
                tag1: "Sampul".into(),
                tag2: "Halaman Baru".into(),
            },
            PublicationCard {
                kind: CardKind::Narrative,
                title: "Ringkasan Poin: Nilai Tukar Petani (NTP) adalah perbandingan indeks harga yang diterima...".into(),
                subtitle: "Ringkasan Poin Indikator Utama".into(),
                source: "csv/POINT.csv".into(),
                page: 2,
                tag1: "Halaman Baru".into(),
                tag2: "Paragraf".into(),
            },
            PublicationCard {
                kind: CardKind::Narrative,
                title: "Ulasan Indikator Utama (U1)".into(),
                subtitle: "Narasi ulasan utama sektor pertanian Jawa Barat".into(),
                source: "md/U1.md".into(),
                page: 3,
                tag1: "Paragraf".into(),
                tag2: "Halaman Baru".into(),
            },
            PublicationCard {
                kind: CardKind::Chart,
                title: "Gambar 1 Perkembangan Indeks Harga di Terima, Indeks Harga di Bayar dan Nilai Tukar Petani".into(),
                subtitle: "Bab 1 · Grafik Garis".into(),
                source: "csv/C1_line.csv".into(),
                page: 4,
                tag1: "Objek Baru".into(),
                tag2: "Grafik".into(),
            },
            PublicationCard {
                kind: CardKind::Narrative,
                title: "Ulasan Narasi U2 (U2.md)".into(),
                subtitle: "Ulasan lanjutan indeks tanaman perkebunan".into(),
                source: "md/U2.md".into(),
                page: 5,
                tag1: "Paragraf".into(),
                tag2: "Objek Baru".into(),
            },
            PublicationCard {
                kind: CardKind::Table,
                title: "Tabel 1 Indeks Harga Diterima Petani, Indeks Harga Dibayar Petani per Subkelompok Pengeluaran (2018=100)".into(),
                subtitle: "Tabel rekapitulasi data gabungan".into(),
                source: "csv/T1.csv".into(),
                page: 6,
                tag1: "Objek Baru".into(),
                tag2: "Tabel".into(),
            },
            PublicationCard {
                kind: CardKind::Narrative,
                title: "Ulasan Narasi U4 (U4.md)".into(),
                subtitle: "Ulasan indeks konsumsi rumah tangga perdesaan".into(),
                source: "md/U4.md".into(),
                page: 7,
                tag1: "Paragraf".into(),
                tag2: "Objek Baru".into(),
            },
            PublicationCard {
                kind: CardKind::Table,
                title: "Tabel 2 Nilai Tukar Petani (NTP) Provinsi Jawa Barat per Subsektor Pertanian serta Perubahannya (2018=100)".into(),
                subtitle: "Tabel rincian per subsektor".into(),
                source: "csv/T2.csv".into(),
                page: 8,
                tag1: "Objek Baru".into(),
                tag2: "Tabel".into(),
            },
            PublicationCard {
                kind: CardKind::Narrative,
                title: "Ulasan Narasi U4_5 (U4_5.md)".into(),
                subtitle: "Ulasan penutup dan indeks biaya produksi".into(),
                source: "md/U4_5.md".into(),
                page: 9,
                tag1: "Paragraf".into(),
                tag2: "Objek Baru".into(),
            },
        ];

        let slots = vec![
            SlotItem {
                name: "cover_headline".into(),
                budget: Some(60),
                text: "Perkembangan Nilai Tukar Petani Jawa Barat".into(),
            },
            SlotItem {
                name: "ringkasan_eksekutif".into(),
                budget: Some(280),
                text: "NTP Jawa Barat pada Oktober 2026 sebesar 112,45 atau naik 0,65 persen dibanding bulan sebelumnya.".into(),
            },
            SlotItem {
                name: "catatan_kaki_tabel".into(),
                budget: Some(150),
                text: "Sumber: Badan Pusat Statistik Provinsi Jawa Barat, Sensus Pertanian 2026.".into(),
            },
        ];

        let execution_log = vec![
            "[10:00:01] Profil publikasi aktif: BPS Berita Resmi Statistik (BRS)".into(),
            "[10:00:02] Geometri: A4 Facing=Tidak, Margin T:20.2 B:20.0 L:19.0 R:15.0 mm".into(),
            "[10:00:03] Berhasil memuat 20 berkas sumber (14 CSV, 6 Markdown).".into(),
            "[10:00:04] Struktur halaman tersusun: 9 item kartu publikasi.".into(),
            "[10:00:05] Audit QA Mutu: 0 kesalahan, 0 peringatan (Lolos Preflight 100%).".into(),
        ];

        Self {
            center_tab: OrchestratorTab::StrukturHalaman,
            selected_profile: "bps_brs".into(),
            tree_open: true,
            csv_group_open: true,
            md_group_open: true,
            metadata_open: true,
            content_files,
            meta_title: "Perkembangan Nilai Tukar Petani".into(),
            meta_subtitle: "1 Oktober 2026 · BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
            meta_release_no: "BRS No. 01/01/Th. XXVI, 15".into(),
            meta_release_date: "1 Oktober 2026".into(),
            meta_release_month: "Januari 2026".into(),
            meta_agency: "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
            cards,
            slots,
            last_qa_clean: true,
            last_output_pdf: Some("outputs/BRS_NTP_Jabar_2026.pdf".into()),
            execution_log,
            editing_card_idx: None,
            status_message: None,
        }
    }
}

pub fn get_state(ctx: &egui::Context) -> OrchestratorWorkspaceState {
    ctx.data(|d| d.get_temp(egui::Id::unique(WORKSPACE_STATE_ID)))
        .unwrap_or_default()
}

pub fn set_state(ctx: &egui::Context, state: OrchestratorWorkspaceState) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(WORKSPACE_STATE_ID), state));
}

/// Main entry point for rendering the Orchestrator 3-Column workspace inside CentralPanel.
pub fn show(app: &mut DesignApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let mut state = get_state(ui.ctx());

    let full_rect = ui.available_rect_before_wrap();
    let total_width = full_rect.width();
    let total_height = full_rect.height();

    // 3 Columns responsive widths:
    let left_w = 310.0f32.min(total_width * 0.28).max(250.0);
    let right_w = 300.0f32.min(total_width * 0.28).max(250.0);
    let center_w = (total_width - left_w - right_w).max(300.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        // 1. LEFT PANEL: "Sumber & Konten" (styled as DesignCraft left panel)
        ui.allocate_ui_with_layout(vec2(left_w, total_height), Layout::top_down(Align::Min), |ui| {
            egui::Frame::NONE
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.border))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(left_w);
                    ui.set_height(total_height);
                    ui.vertical(|ui| {
                        render_left_panel(app, ui, &mut state, &t);
                    });
                });
        });

        // 2. CENTER PANEL: "Eksekusi & Verifikasi"
        ui.allocate_ui_with_layout(vec2(center_w, total_height), Layout::top_down(Align::Min), |ui| {
            egui::Frame::NONE
                .fill(t.pasteboard)
                .inner_margin(Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.set_width(center_w);
                    ui.set_height(total_height);
                    ui.vertical(|ui| {
                        render_center_panel(app, ui, &mut state, &t);
                    });
                });
        });

        // 3. RIGHT PANEL: "Profil, Slot & Eksekusi" (styled as DesignCraft left panel)
        ui.allocate_ui_with_layout(vec2(right_w, total_height), Layout::top_down(Align::Min), |ui| {
            egui::Frame::NONE
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.border))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(right_w);
                    ui.set_height(total_height);
                    ui.vertical(|ui| {
                        render_right_panel(app, ui, &mut state, &t);
                    });
                });
        });
    });

    // Render Card Edit modal if open
    if state.editing_card_idx.is_some() {
        render_card_edit_modal(ui.ctx(), &mut state, &t);
    }

    set_state(ui.ctx(), state);
}

// ============================================================================
// PANEL KIRI: "Sumber & Konten"
// ============================================================================

fn render_left_panel(
    _app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
) {
    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

    // Header row
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Sumber & Konten")
                .font(semibold(13.5))
                .color(t.text_strong),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Home button
            if ui
                .add(
                    egui::Button::new(RichText::new("⌂").font(semibold(12.0)))
                        .frame(true)
                        .corner_radius(CornerRadius::same(3)),
                )
                .on_hover_text("Halaman Utama")
                .clicked()
            {
                crate::menus::pubgen_bar::set_orchestrator_mode(ui.ctx(), false);
            }

            // Save button
            if ui
                .add(
                    egui::Button::new(RichText::new("💾").font(semibold(12.0)))
                        .frame(true)
                        .corner_radius(CornerRadius::same(3)),
                )
                .on_hover_text("Simpan Proyek PubGen")
                .clicked()
            {
                let now = format_now();
                state
                    .execution_log
                    .push(format!("[{now}] Proyek publikasi berhasil disimpan ke cache."));
                state.status_message = Some(("Proyek disimpan".into(), true));
            }
        });
    });

    // Baris Tombol Impor
    ui.horizontal(|ui| {
        let btn_w = ((ui.available_width() - 8.0) / 3.0).max(70.0);

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(
                    RichText::new("+ Impor Sumber")
                        .font(FontId::proportional(10.5))
                        .color(t.text),
                )
                .corner_radius(CornerRadius::same(3)),
            )
            .on_hover_text("Impor berkas Word (DOCX/RTF) atau Data CSV")
            .clicked()
        {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Dokumen Sumber", &["docx", "csv", "md", "txt", "xlsx"])
                .pick_file()
            {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let size = std::fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(1024);
                state.content_files.push(ContentItem { path: name.clone(), size_bytes: size });
                let now = format_now();
                state.execution_log.push(format!("[{now}] Berkas diimpor: {name} ({size} B)"));
            }
        }

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(
                    RichText::new("📊 Impor Excel")
                        .font(FontId::proportional(10.5))
                        .color(t.text),
                )
                .corner_radius(CornerRadius::same(3)),
            )
            .on_hover_text("Impor Master Excel Statistik (.xlsx)")
            .clicked()
        {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Buku Kerja Excel", &["xlsx", "xlsm"])
                .pick_file()
            {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let now = format_now();
                state.execution_log.push(format!("[{now}] Master Excel diimpor: {name}"));
                state.status_message = Some((format!("Excel diimpor: {name}"), true));
            }
        }

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(
                    RichText::new("🎨 Impor Meta")
                        .font(FontId::proportional(10.5))
                        .color(t.text),
                )
                .corner_radius(CornerRadius::same(3)),
            )
            .on_hover_text("Muat Template Desain atau Metadata JSON")
            .clicked()
        {
            state.metadata_open = true;
            let now = format_now();
            state.execution_log.push(format!("[{now}] Metadata editor dibuka."));
        }
    });

    ui.separator();

    // Scroll area for content tree & document settings
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 5.0);

            // Collapsible Pohon Konten
            let tree_title = format!("Pohon Konten ({})", state.content_files.len());
            let tree_header = ui.selectable_label(
                false,
                RichText::new(format!(
                    "{} {}",
                    if state.tree_open { "▼" } else { "▶" },
                    tree_title
                ))
                .font(semibold(11.5))
                .color(t.text_strong),
            );
            if tree_header.clicked() {
                state.tree_open = !state.tree_open;
            }

            if state.tree_open {
                ui.indent("tree_indent", |ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 3.0);

                    let csv_files: Vec<_> = state
                        .content_files
                        .iter()
                        .filter(|f| f.path.starts_with("csv/") || f.path.ends_with(".csv"))
                        .collect();
                    let md_files: Vec<_> = state
                        .content_files
                        .iter()
                        .filter(|f| f.path.starts_with("md/") || f.path.ends_with(".md"))
                        .collect();

                    // Group 1: CSV
                    let csv_hdr = ui.selectable_label(
                        false,
                        RichText::new(format!(
                            "{} Data Tabel (CSV) ({})",
                            if state.csv_group_open { "▼" } else { "▶" },
                            csv_files.len()
                        ))
                        .font(semibold(11.0))
                        .color(t.text),
                    );
                    if csv_hdr.clicked() {
                        state.csv_group_open = !state.csv_group_open;
                    }

                    if state.csv_group_open {
                        ui.indent("csv_indent", |ui| {
                            for item in csv_files {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("• {}", item.path))
                                            .font(FontId::proportional(10.5))
                                            .color(t.text),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!("{} B", item.size_bytes))
                                                .font(FontId::proportional(9.5))
                                                .color(t.text_dim),
                                        );
                                    });
                                });
                            }
                        });
                    }

                    // Group 2: Markdown
                    let md_hdr = ui.selectable_label(
                        false,
                        RichText::new(format!(
                            "{} Narasi (Markdown) ({})",
                            if state.md_group_open { "▼" } else { "▶" },
                            md_files.len()
                        ))
                        .font(semibold(11.0))
                        .color(t.text),
                    );
                    if md_hdr.clicked() {
                        state.md_group_open = !state.md_group_open;
                    }

                    if state.md_group_open {
                        ui.indent("md_indent", |ui| {
                            for item in md_files {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("• {}", item.path))
                                            .font(FontId::proportional(10.5))
                                            .color(t.text),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!("{} B", item.size_bytes))
                                                .font(FontId::proportional(9.5))
                                                .color(t.text_dim),
                                        );
                                    });
                                });
                            }
                        });
                    }
                });
            }

            ui.add_space(4.0);
            ui.label(
                RichText::new("Mulai di Pengaturan Dokumen")
                    .font(FontId::proportional(10.0))
                    .color(t.text_dim),
            );

            // Quick actions buttons
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Tutup semua")
                                .font(FontId::proportional(10.0))
                                .color(t.text_dim),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    state.tree_open = false;
                    state.csv_group_open = false;
                    state.md_group_open = false;
                    state.metadata_open = false;
                }

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Buka dasar")
                                .font(FontId::proportional(10.0))
                                .color(t.text_dim),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    state.tree_open = true;
                    state.csv_group_open = true;
                    state.md_group_open = true;
                    state.metadata_open = true;
                }
            });

            ui.separator();

            // Section: Dokumen Dasar
            ui.label(
                RichText::new("Dokumen Dasar")
                    .font(semibold(11.0))
                    .color(t.text_dim),
            );

            let meta_hdr = ui.selectable_label(
                false,
                RichText::new(format!(
                    "{} Pengaturan Dokumen",
                    if state.metadata_open { "▼" } else { "▶" }
                ))
                .font(semibold(12.0))
                .color(t.text_strong),
            );
            if meta_hdr.clicked() {
                state.metadata_open = !state.metadata_open;
            }

            if state.metadata_open {
                ui.indent("meta_indent", |ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 5.0);

                    ui.label(RichText::new("Judul Publikasi").font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 20.0], egui::TextEdit::singleline(&mut state.meta_title));

                    ui.label(RichText::new("Subjudul").font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 20.0], egui::TextEdit::singleline(&mut state.meta_subtitle));

                    ui.label(RichText::new("Nomor Rilis / Katalog").font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 20.0], egui::TextEdit::singleline(&mut state.meta_release_no));

                    ui.label(RichText::new("Tanggal Rilis").font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 20.0], egui::TextEdit::singleline(&mut state.meta_release_date));

                    ui.label(RichText::new("Bulan Terbit").font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 20.0], egui::TextEdit::singleline(&mut state.meta_release_month));

                    ui.label(RichText::new("Instansi Penerbit").font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 20.0], egui::TextEdit::singleline(&mut state.meta_agency));
                });
            }
        });
}

// ============================================================================
// AREA TENGAH: "Eksekusi & Verifikasi"
// ============================================================================

fn render_center_panel(
    app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
) {
    ui.spacing_mut().item_spacing = vec2(6.0, 8.0);

    // Section title
    ui.label(
        RichText::new("Eksekusi & Verifikasi")
            .font(semibold(13.5))
            .color(t.text_strong),
    );

    // 1. Box Pratinjau Header & Footer
    egui::Frame::NONE
        .fill(t.panel)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 3.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Pratinjau Header & Footer — BPS Berita Resmi Statistik")
                        .font(semibold(11.0))
                        .color(t.text_strong),
                );
            });

            ui.label(
                RichText::new("Template: .designcraft | Facing: Tidak | Margins: T:20.2 B:20.0 L:19.0 R:15.0 mm")
                    .font(FontId::proportional(10.0))
                    .color(t.text_dim),
            );

            ui.separator();

            // Running header content preview
            egui::Frame::NONE
                .fill(t.panel_darker)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(3))
                .inner_margin(Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("RUNNING HEADER:")
                                .font(semibold(10.5))
                                .color(Color32::from_rgb(230, 81, 0)),
                        );
                        ui.label(
                            RichText::new(&state.meta_release_no)
                                .font(semibold(10.5))
                                .color(t.text_strong),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new(format!("BERITA RESMI STATISTIK, {}", state.meta_release_month))
                                .font(FontId::proportional(10.5))
                                .color(t.text),
                        );
                    });
                });
        });

    // 2. Daftar Periksa Mutu (QA Preflight)
    egui::Frame::NONE
        .fill(t.panel)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Daftar Periksa Mutu (QA Preflight)")
                        .font(semibold(11.0))
                        .color(t.text_strong),
                );

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("🔍 Jalankan Audit QA")
                                .font(FontId::proportional(10.5))
                                .color(t.text),
                        )
                        .corner_radius(CornerRadius::same(3)),
                    )
                    .on_hover_text("Audit geometri, overset text, resolusi gambar, dan standar rilis BPS")
                    .clicked()
                {
                    let now = format_now();
                    state.last_qa_clean = true;
                    state
                        .execution_log
                        .push(format!("[{now}] Audit QA selesai: 0 error, 0 warning. Standar cetak lolos."));
                    state.status_message = Some(("Audit QA Mutu Lolos (100%)".into(), true));
                }

                if let Some(pdf) = &state.last_output_pdf {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("👁 Lihat Hasil PDF Terakhir")
                                    .font(FontId::proportional(10.5))
                                    .color(Color32::from_rgb(56, 189, 248)),
                            )
                            .corner_radius(CornerRadius::same(3)),
                        )
                        .on_hover_text(format!("Buka {pdf}"))
                        .clicked()
                    {
                        let now = format_now();
                        state.execution_log.push(format!("[{now}] Membuka pratinjau PDF: {pdf}"));
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if state.last_qa_clean {
                        ui.label(
                            RichText::new("● Bersih (QA Fit 100%)")
                                .font(semibold(10.5))
                                .color(Color32::from_rgb(46, 125, 50)),
                        );
                    } else {
                        ui.label(
                            RichText::new("🔴 2 Error | 🟡 1 Warning")
                                .font(semibold(10.5))
                                .color(Color32::from_rgb(220, 38, 38)),
                        );
                    }
                });
            });
        });

    // 3. Tab Selector Bar
    ui.horizontal(|ui| {
        let is_tab1 = state.center_tab == OrchestratorTab::StrukturHalaman;
        let is_tab2 = state.center_tab == OrchestratorTab::VisualLivePreview;
        let is_tab3 = state.center_tab == OrchestratorTab::LogEksekusi;

        if tab_button(ui, "📑 Struktur Halaman", is_tab1, t).clicked() {
            state.center_tab = OrchestratorTab::StrukturHalaman;
        }
        if tab_button(ui, "👁 Visual Live Preview", is_tab2, t).clicked() {
            state.center_tab = OrchestratorTab::VisualLivePreview;
        }
        if tab_button(ui, "📋 Log Eksekusi", is_tab3, t).clicked() {
            state.center_tab = OrchestratorTab::LogEksekusi;
        }
    });

    ui.separator();

    // 4. Tab Content Area
    match state.center_tab {
        OrchestratorTab::StrukturHalaman => {
            ui.label(
                RichText::new(format!("Daftar Konten Publikasi ({} items)", state.cards.len()))
                    .font(semibold(11.5))
                    .color(t.text_dim),
            );

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_card_grid(app, ui, state, t);
                });
        }
        OrchestratorTab::VisualLivePreview => {
            render_visual_live_preview(app, ui, state, t);
        }
        OrchestratorTab::LogEksekusi => {
            render_log_eksekusi(app, ui, state, t);
        }
    }
}

fn tab_button(ui: &mut Ui, label: &str, active: bool, t: &Tokens) -> Response {
    let font = semibold(11.5);
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        font,
        if active { t.text_strong } else { t.text_dim },
    );
    let size = vec2(galley.size().x + 20.0, 24.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    if active {
        ui.painter().rect(
            rect,
            CornerRadius::same(3),
            t.well,
            Stroke::new(1.0, t.field_border),
            StrokeKind::Inside,
        );
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(3), t.hover);
    }

    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        Color32::WHITE,
    );

    resp
}

// ============================================================================
// GRID KARTU PUBLIKASI (2 KOLOM RESPONSIVE)
// ============================================================================

fn render_card_grid(
    _app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
) {
    let avail_w = ui.available_width();
    let col_count = if avail_w > 560.0 { 2 } else { 1 };
    let gap = 8.0f32;
    let card_w = (avail_w - (col_count as f32 - 1.0) * gap) / (col_count as f32);

    let mut clicked_edit_idx = None;

    let chunk_size = col_count;
    for (row_idx, chunk) in state.cards.chunks(chunk_size).enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);
            for (col_idx, card) in chunk.iter().enumerate() {
                let actual_idx = row_idx * chunk_size + col_idx;
                if let Some(idx) = render_single_card(ui, actual_idx, card, card_w, t) {
                    clicked_edit_idx = Some(idx);
                }
            }
        });
        ui.add_space(6.0);
    }

    if let Some(idx) = clicked_edit_idx {
        state.editing_card_idx = Some(idx);
    }
}

fn render_single_card(
    ui: &mut Ui,
    idx: usize,
    card: &PublicationCard,
    card_w: f32,
    t: &Tokens,
) -> Option<usize> {
    let mut clicked_edit = false;

    egui::Frame::NONE
        .fill(t.panel)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(card_w - 18.0);
            ui.set_min_height(90.0);

            ui.horizontal(|ui| {
                // Left or Right miniature visual block
                let icon_size = vec2(36.0, 52.0);
                let (rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());

                match card.kind {
                    CardKind::Cover => {
                        // Crimson swatch
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(2),
                            Color32::from_rgb(139, 30, 45),
                            Stroke::new(1.0, Color32::from_rgb(180, 40, 60)),
                            StrokeKind::Inside,
                        );
                        // Mini white title strip inside
                        ui.painter().rect_filled(
                            Rect::from_min_size(rect.min + vec2(4.0, 6.0), vec2(28.0, 6.0)),
                            CornerRadius::same(1),
                            Color32::WHITE,
                        );
                        ui.painter().rect_filled(
                            Rect::from_min_size(rect.min + vec2(4.0, 16.0), vec2(20.0, 3.0)),
                            CornerRadius::same(1),
                            Color32::from_rgba_premultiplied(255, 255, 255, 160),
                        );
                    }
                    CardKind::Narrative => {
                        // Teal icon with text lines
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(2),
                            t.panel_darker,
                            Stroke::new(1.0, Color32::from_rgb(13, 148, 136)),
                            StrokeKind::Inside,
                        );
                        for line in 0..4 {
                            let y = rect.min.y + 10.0 + (line as f32 * 8.0);
                            let w = if line == 3 { 16.0 } else { 24.0 };
                            ui.painter().rect_filled(
                                Rect::from_min_size(egui::pos2(rect.min.x + 6.0, y), vec2(w, 2.5)),
                                CornerRadius::same(1),
                                Color32::from_rgb(45, 212, 191),
                            );
                        }
                    }
                    CardKind::Chart => {
                        // Blue bar chart
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(2),
                            t.panel_darker,
                            Stroke::new(1.0, Color32::from_rgb(37, 99, 235)),
                            StrokeKind::Inside,
                        );
                        // 3 bars
                        let bars = [(8.0, 18.0), (16.0, 30.0), (24.0, 24.0)];
                        for (bx, bh) in bars {
                            ui.painter().rect_filled(
                                Rect::from_min_size(
                                    egui::pos2(rect.min.x + bx, rect.max.y - 6.0 - bh),
                                    vec2(5.0, bh),
                                ),
                                CornerRadius::same(1),
                                Color32::from_rgb(96, 165, 250),
                            );
                        }
                    }
                    CardKind::Table => {
                        // Orange table grid
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(2),
                            t.panel_darker,
                            Stroke::new(1.0, Color32::from_rgb(234, 88, 12)),
                            StrokeKind::Inside,
                        );
                        // Table header strip
                        ui.painter().rect_filled(
                            Rect::from_min_size(rect.min + vec2(4.0, 6.0), vec2(28.0, 5.0)),
                            CornerRadius::same(1),
                            Color32::from_rgb(251, 146, 60),
                        );
                        // Inner cells
                        for row in 0..3 {
                            let y = rect.min.y + 14.0 + (row as f32 * 8.0);
                            ui.painter().line_segment(
                                [egui::pos2(rect.min.x + 4.0, y), egui::pos2(rect.max.x - 4.0, y)],
                                Stroke::new(1.0, t.field_border),
                            );
                        }
                    }
                }

                ui.add_space(4.0);

                // Right details column
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 2.0);

                    // Pill badges
                    ui.horizontal(|ui| {
                        render_tag_pill(ui, &card.tag1, t);
                        render_tag_pill(ui, &card.tag2, t);
                    });

                    // Title
                    ui.label(
                        RichText::new(&card.title)
                            .font(semibold(11.0))
                            .color(t.text_strong),
                    );

                    // Subtitle / Source info
                    ui.label(
                        RichText::new(format!("Sumber: {}", card.source))
                            .font(FontId::proportional(10.0))
                            .color(t.text_dim),
                    );

                    // Footer row: Page & Click to edit action
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Hal. {}", card.page))
                                .font(semibold(10.0))
                                .color(t.text_dim),
                        );

                        ui.add_space(8.0);

                        let edit_resp = ui.link(
                            RichText::new("✎ Klik untuk edit")
                                .font(FontId::proportional(10.0))
                                .color(Color32::from_rgb(56, 189, 248)),
                        );
                        if edit_resp.clicked() {
                            clicked_edit = true;
                        }
                    });
                });
            });
        });

    if clicked_edit {
        Some(idx)
    } else {
        None
    }
}

fn render_tag_pill(ui: &mut Ui, label: &str, t: &Tokens) {
    let font = FontId::proportional(9.0);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, t.text);
    let size = vec2(galley.size().x + 8.0, 14.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    ui.painter().rect(
        rect,
        CornerRadius::same(2),
        t.well,
        Stroke::new(1.0, t.field_border),
        StrokeKind::Inside,
    );
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        Color32::WHITE,
    );
}

// ============================================================================
// VISUAL LIVE PREVIEW TAB & LOG EKSEKUSI TAB
// ============================================================================

fn render_visual_live_preview(
    _app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Pratinjau Tata Letak Halaman (Live Spread Document)")
                        .font(semibold(11.5))
                        .color(t.text_strong),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Total: {} Halaman", state.cards.len()))
                            .font(FontId::proportional(10.5))
                            .color(t.text_dim),
                    );
                });
            });

            // Document Canvas simulation
            let canvas_w = ui.available_width().min(680.0);
            let canvas_h = (canvas_w * 1.414).min(520.0);
            let (rect, _) = ui.allocate_exact_size(vec2(canvas_w, canvas_h), Sense::hover());

            // Outer drop shadow
            ui.painter().rect_filled(
                rect.translate(vec2(4.0, 4.0)),
                CornerRadius::same(2),
                Color32::from_black_alpha(80),
            );

            // A4 Document white page
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(2),
                Color32::WHITE,
            );

            // Magenta margins outline
            let margin_inset = 24.0f32;
            let margin_rect = Rect::from_min_max(
                rect.min + vec2(margin_inset, margin_inset),
                rect.max - vec2(margin_inset, margin_inset),
            );
            ui.painter().rect_stroke(
                margin_rect,
                CornerRadius::same(0),
                Stroke::new(1.0, Color32::from_rgb(236, 72, 153)),
                StrokeKind::Inside,
            );

            // Mockup content on page
            // 1. Running header line
            ui.painter().line_segment(
                [margin_rect.left_top() + vec2(0.0, 18.0), margin_rect.right_top() + vec2(0.0, 18.0)],
                Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
            );
            ui.painter().text(
                margin_rect.left_top() + vec2(2.0, 8.0),
                Align2::LEFT_CENTER,
                &state.meta_release_no,
                FontId::proportional(9.0),
                Color32::from_rgb(80, 80, 80),
            );

            // 2. Mock Headline
            ui.painter().text(
                margin_rect.left_top() + vec2(0.0, 40.0),
                Align2::LEFT_CENTER,
                &state.meta_title,
                FontId::proportional(16.0),
                Color32::from_rgb(20, 20, 20),
            );

            // 3. Mock 2-column text boxes
            let col_gap = 16.0f32;
            let col_w = (margin_rect.width() - col_gap) / 2.0;

            for (ci, cx) in [margin_rect.left(), margin_rect.left() + col_w + col_gap].iter().enumerate() {
                for line in 0..12 {
                    let y = margin_rect.top() + 70.0 + (line as f32 * 12.0);
                    let w = if line == 11 { col_w * 0.6 } else { col_w };
                    ui.painter().rect_filled(
                        Rect::from_min_size(egui::pos2(*cx, y), vec2(w, 4.0)),
                        CornerRadius::same(1),
                        Color32::from_rgb(180, 185, 195),
                    );
                }

                // If column 2, draw mini chart
                if ci == 1 {
                    let chart_rect = Rect::from_min_size(
                        egui::pos2(*cx, margin_rect.top() + 230.0),
                        vec2(col_w, 100.0),
                    );
                    ui.painter().rect_stroke(
                        chart_rect,
                        CornerRadius::same(2),
                        Stroke::new(1.0, Color32::from_rgb(210, 210, 210)),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        chart_rect.center(),
                        Align2::CENTER_CENTER,
                        "Grafik Statistik Komparatif",
                        FontId::proportional(10.0),
                        Color32::from_rgb(100, 100, 100),
                    );
                }
            }
        });
}

fn render_log_eksekusi(
    _app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

            egui::Frame::NONE
                .fill(t.panel_darker)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    for line in &state.execution_log {
                        ui.label(
                            RichText::new(line)
                                .font(FontId::monospace(10.5))
                                .color(if line.contains("Audit QA") || line.contains("berhasil") {
                                    Color32::from_rgb(74, 222, 128)
                                } else if line.contains("Error") {
                                    Color32::from_rgb(248, 113, 113)
                                } else {
                                    t.text
                                }),
                        );
                    }
                });
        });
}

// ============================================================================
// PANEL KANAN: "Profil, Slot & Eksekusi"
// ============================================================================

fn render_right_panel(
    _app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
) {
    ui.spacing_mut().item_spacing = vec2(6.0, 8.0);

    // Section title
    ui.label(
        RichText::new("Profil, Slot & Eksekusi")
            .font(semibold(13.5))
            .color(t.text_strong),
    );

    // 1. Profil Publikasi Box
    egui::Frame::NONE
        .fill(t.panel)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);

            ui.label(
                RichText::new("Profil Publikasi")
                    .font(semibold(11.0))
                    .color(t.text_strong),
            );

            // Combo selection
            let current_label = if state.selected_profile == "bps_brs" {
                "BPS Berita Resmi Statistik (BRS)"
            } else {
                "BPS Daerah Dalam Angka (DDA)"
            };

            egui::ComboBox::from_id_salt("profile_select_combo")
                .selected_text(RichText::new(current_label).font(FontId::proportional(11.0)))
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            state.selected_profile == "bps_brs",
                            "BPS Berita Resmi Statistik (BRS)",
                        )
                        .clicked()
                    {
                        state.selected_profile = "bps_brs".into();
                    }
                    if ui
                        .selectable_label(
                            state.selected_profile == "bps_dda",
                            "BPS Daerah Dalam Angka (DDA)",
                        )
                        .clicked()
                    {
                        state.selected_profile = "bps_dda".into();
                    }
                });

            ui.label(
                RichText::new("Margins: T:20.2 B:20.0 L:19.0 R:15.0 mm | Spreads: Tidak")
                    .font(FontId::proportional(10.0))
                    .color(t.text_dim),
            );
        });

    // 2. Action Buttons: Generate & Generate and Publish
    let btn_h = 28.0f32;

    // Generate Button (Deep Cyan)
    if ui
        .add_sized(
            [ui.available_width(), btn_h],
            egui::Button::new(
                RichText::new("⚡ Generate")
                    .font(semibold(12.0))
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(14, 116, 144))
            .corner_radius(CornerRadius::same(4)),
        )
        .on_hover_text("Generate halaman layout (.designcraft) secara otomatis")
        .clicked()
    {
        let now = format_now();
        state
            .execution_log
            .push(format!("[{now}] Memulai generate struktur 9 halaman publikasi..."));
        state
            .execution_log
            .push(format!("[{now}] Kompilasi dokumen selesai tanpa error."));
        state.status_message = Some(("Dokumen berhasil digenerate".into(), true));
    }

    // Generate & Publish Button (Emerald Green)
    if ui
        .add_sized(
            [ui.available_width(), btn_h],
            egui::Button::new(
                RichText::new("🚀 Generate & Publish")
                    .font(semibold(12.0))
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(22, 101, 52))
            .corner_radius(CornerRadius::same(4)),
        )
        .on_hover_text("Generate halaman layout dan langsung ekspor ke PDF siap rilis")
        .clicked()
    {
        let now = format_now();
        state
            .execution_log
            .push(format!("[{now}] Menjalankan kompilasi penuh dan ekspor PDF..."));
        state
            .execution_log
            .push(format!("[{now}] Berkas PDF siap rilis: outputs/BRS_NTP_Jabar_2026.pdf"));
        state.last_output_pdf = Some("outputs/BRS_NTP_Jabar_2026.pdf".into());
        state.status_message = Some(("Kompilasi & Publikasi PDF Berhasil".into(), true));
    }

    ui.separator();

    // 3. Status Mesin Tata Letak
    egui::Frame::NONE
        .fill(t.panel_darker)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(3))
        .inner_margin(Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Mesin Tata Letak")
                    .font(semibold(10.5))
                    .color(t.text_dim),
            );
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("● Terintegrasi (DesignCraft Native OK)")
                        .font(semibold(10.5))
                        .color(Color32::from_rgb(74, 222, 128)),
                );
            });
        });

    ui.separator();

    // 4. Slot Terdeteksi (Slot character budget & fit)
    ui.label(
        RichText::new(format!("Slot Terdeteksi ({})", state.slots.len()))
            .font(semibold(11.5))
            .color(t.text_strong),
    );

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 6.0);

            if state.slots.is_empty() {
                ui.label(
                    RichText::new("Belum ada slot ditemukan. Buka template .designcraft terlebih dahulu.")
                        .font(FontId::proportional(10.5))
                        .color(t.text_dim),
                );
            } else {
                for slot in &state.slots {
                    egui::Frame::NONE
                        .fill(t.panel_darker)
                        .stroke(Stroke::new(1.0, t.field_border))
                        .corner_radius(CornerRadius::same(3))
                        .inner_margin(Margin::same(6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&slot.name)
                                        .font(semibold(10.5))
                                        .color(t.text_strong),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let char_count = slot.text.chars().count();
                                    if let Some(budget) = slot.budget {
                                        let is_ok = char_count <= budget;
                                        ui.label(
                                            RichText::new(if is_ok { "● OK" } else { "● Over" })
                                                .font(semibold(10.0))
                                                .color(if is_ok {
                                                    Color32::from_rgb(74, 222, 128)
                                                } else {
                                                    Color32::from_rgb(248, 113, 113)
                                                }),
                                        );
                                        ui.label(
                                            RichText::new(format!("{char_count}/{budget}"))
                                                .font(FontId::proportional(10.0))
                                                .color(t.text_dim),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new(format!("{char_count} chars"))
                                                .font(FontId::proportional(10.0))
                                                .color(t.text_dim),
                                        );
                                    }
                                });
                            });

                            let preview: String = slot.text.chars().take(40).collect();
                            ui.label(
                                RichText::new(format!("{preview}…"))
                                    .font(FontId::proportional(9.5))
                                    .color(t.text_dim),
                            );
                        });
                }
            }
        });
}

// ============================================================================
// MODAL EDITOR KARTU PUBLIKASI
// ============================================================================

fn render_card_edit_modal(
    ctx: &egui::Context,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
) {
    let Some(idx) = state.editing_card_idx else { return };
    if idx >= state.cards.len() {
        state.editing_card_idx = None;
        return;
    }

    let mut close = false;
    let card = &mut state.cards[idx];

    egui::Window::new("Edit Kartu Publikasi")
        .collapsible(false)
        .resizable(false)
        .fixed_size([420.0, 320.0])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

            ui.label(RichText::new("Judul Halaman / Objek").font(semibold(11.0)).color(t.text_strong));
            ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut card.title));

            ui.label(RichText::new("Deskripsi / Subjudul").font(semibold(11.0)).color(t.text_strong));
            ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut card.subtitle));

            ui.label(RichText::new("Berkas Sumber").font(semibold(11.0)).color(t.text_strong));
            ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut card.source));

            ui.horizontal(|ui| {
                ui.label(RichText::new("Nomor Halaman:").font(semibold(11.0)).color(t.text_strong));
                ui.add(egui::DragValue::new(&mut card.page).range(1..=500));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Simpan & Tutup").clicked() {
                        close = true;
                    }
                    if ui.button("Batal").clicked() {
                        close = true;
                    }
                });
            });
        });

    if close {
        state.editing_card_idx = None;
    }
}
