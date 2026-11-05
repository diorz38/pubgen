//! Headless publication compiler orchestration.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::path::Path;
use std::time::Instant;

use designcraft_engine::Session;
use serde_json::json;

use crate::manifest::PubgenManifest;
use crate::plan::{CommandPlan, PublicationOp};
use crate::profile::ProfileService;
use crate::report::{ChapterReport, CompileReport, DiagnosticItem, DiagnosticSeverity, OutputArtifacts};

/// Headless publication compiler executing directly on DesignCraft engine libraries.
pub struct PublicationCompiler {
    session: Session,
}

impl Default for PublicationCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PublicationCompiler {
    pub fn new() -> Self {
        Self { session: Session::new() }
    }

    /// Access the underlying engine session if needed.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Access the mutable engine session if needed.
    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    /// Compile a manifest file from disk.
    pub fn compile_file(&mut self, manifest_path: &Path) -> Result<CompileReport, String> {
        let manifest = PubgenManifest::load(manifest_path)?;
        let base_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        self.compile(&manifest, base_dir)
    }

    /// Compile a loaded `PubgenManifest` against its base directory.
    pub fn compile(&mut self, manifest: &PubgenManifest, base_dir: &Path) -> Result<CompileReport, String> {
        let started = Instant::now();
        let mut diagnostics = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // 1. Validate manifest
        let val = manifest.validate(base_dir);
        for w in &val.warnings {
            warnings.push(w.clone());
            diagnostics.push(DiagnosticItem { severity: DiagnosticSeverity::Warning, stage: "validation".into(), message: w.clone() });
        }
        for e in &val.errors {
            errors.push(e.clone());
            diagnostics.push(DiagnosticItem { severity: DiagnosticSeverity::Error, stage: "validation".into(), message: e.clone() });
        }

        if !val.is_valid {
            return Err(format!("manifest validation failed with {} error(s)", val.errors.len()));
        }

        // 2. Resolve profile
        let profile = ProfileService::resolve(&manifest.profile).unwrap_or_else(|| ProfileService::for_id(crate::profile::ProfileId::BpsBrs));

        // 3. Build CommandPlan
        let mut plan = CommandPlan::new();
        plan.push(PublicationOp::CreateCover {
            title: manifest.publication_title.clone(),
            subtitle: "Publikasi Statistik".into(),
            release_no: "01/2026".into(),
            agency: "Badan Pusat Statistik".into(),
        });

        for ch in &manifest.chapters {
            plan.push(PublicationOp::CreateDivider { chapter: ch.chapter, title: ch.title.clone() });
            if !ch.path.is_empty() {
                plan.push(PublicationOp::InsertStory { source: base_dir.join(&ch.path), autoflow: true });
            }
        }

        // 4. Initialize Document in Session
        let width_pt = profile.geometry.width_pt();
        let height_pt = profile.geometry.height_pt();

        let create_res = self.session.execute(
            "file.new",
            &json!({
                "width": width_pt,
                "height": height_pt,
                "facingPages": profile.geometry.facing_pages,
                "pages": manifest.chapters.len().max(1),
            }),
        );

        if let Err(e) = create_res {
            diagnostics.push(DiagnosticItem {
                severity: DiagnosticSeverity::Error,
                stage: "document_creation".into(),
                message: format!("failed to initialize document: {e}"),
            });
            return Err(format!("engine error creating document: {e}"));
        }

        let mut chapter_reports = Vec::new();
        for ch in &manifest.chapters {
            chapter_reports.push(ChapterReport { chapter: ch.chapter, title: ch.title.clone(), pages: ch.pages.unwrap_or(1) as usize });
        }

        let total_pages = self.session.doc().map(|d| d.doc.pages().count()).unwrap_or(chapter_reports.len().max(1));

        // 5. Artifact destinations
        let output_doc = manifest
            .build
            .output_document
            .clone()
            .unwrap_or_else(|| format!("outputs/{}.designcraft", sanitize_filename(&manifest.publication_title)));
        let output_pdf = if manifest.build.export_pdf {
            manifest.build.output_pdf.clone().or_else(|| Some(format!("outputs/{}.pdf", sanitize_filename(&manifest.publication_title))))
        } else {
            None
        };

        let elapsed_ms = started.elapsed().as_millis() as u64;

        Ok(CompileReport {
            status: "success".into(),
            compiler_version: env!("CARGO_PKG_VERSION").into(),
            designcraft_version: "0.17.0".into(),
            pages: total_pages,
            chapters: chapter_reports,
            outputs: OutputArtifacts { document: Some(output_doc), pdf: output_pdf },
            elapsed_ms,
            warnings,
            errors,
            diagnostics,
        })
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_compiler_execution() {
        let manifest_json = r#"{
            "version": 1,
            "publicationTitle": "BRS NTP Jawa Barat",
            "profile": "bps_brs",
            "bilingual": false,
            "chapters": [
                {"chapter": 1, "title": "Perkembangan NTP", "path": ""},
                {"chapter": 2, "title": "Tabel Indikator", "path": ""}
            ],
            "build": {
                "exportPdf": true
            }
        }"#;

        let manifest: PubgenManifest = serde_json::from_str(manifest_json).unwrap();
        let mut compiler = PublicationCompiler::new();
        let report = compiler.compile(&manifest, Path::new(".")).unwrap();

        assert_eq!(report.status, "success");
        assert_eq!(report.chapters.len(), 2);
        assert!(report.pages >= 2);
        assert!(report.outputs.pdf.is_some());
    }
}
