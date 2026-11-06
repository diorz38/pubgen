use pubgen_core::{open_pubgen, save_pubgen, export_workbook, ProjectContent, WorkspaceDraft};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct OrchestratorProject {
    pub path: Option<PathBuf>,
    pub draft: Option<WorkspaceDraft>,
    pub status: String,
    pub dirty: bool,
    pub ui_synced: bool,
    pub md: Vec<(String, Vec<u8>)>,
    pub assets: Vec<(String, Vec<u8>)>,
}

impl OrchestratorProject {
    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        let opened = open_pubgen(path).map_err(|error| error.to_string())?;
        let csv = opened.content.csv;
        let md = opened.content.md.clone();
        let assets = opened.content.assets.clone();
        let metadata = opened.metadata;
        self.draft = Some(WorkspaceDraft { manifest: opened.manifest, metadata: metadata.clone(), glossary: opened.glossary, csv, warnings: Vec::new() });
        self.md = md;
        self.assets = assets;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        self.status = "Project opened".into();
        Ok(())
    }
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let draft = self.draft.as_ref().ok_or_else(|| "No workspace loaded".to_string())?;
        let content = ProjectContent { csv: draft.csv.clone(), ..ProjectContent::default() };
        save_pubgen(path, &draft.manifest, &draft.metadata, &draft.glossary, &content).map_err(|error| error.to_string())?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        self.status = "Project saved".into();
        Ok(())
    }
    pub fn export_xlsx(&mut self, path: &Path) -> Result<(), String> {
        let draft = self.draft.as_ref().ok_or_else(|| "No workspace loaded".to_string())?;
        export_workbook(draft, path).map_err(|error| error.to_string())?;
        self.status = "Workbook exported".into();
        Ok(())
    }
    pub fn sync_ui(&mut self, ctx: &egui::Context) {
        if let Some(draft) = &self.draft {
            designcraft_ui_egui::dock::pubgen::workspace::sync_external_workspace(ctx, self.path.as_ref().and_then(|path| path.to_str()).unwrap_or("Workbook"), &draft.metadata, &draft.csv, &self.md, &self.assets);
            self.ui_synced = true;
        }
    }
    pub fn clear(&mut self) { *self = Self::default(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_project_rejects_save() { let mut project = OrchestratorProject::default(); assert!(project.save(Path::new("x.pubgen")).is_err()); }
}
