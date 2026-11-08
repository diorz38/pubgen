use designcraft_ui_egui::dock::pubgen::workspace::{self, OrchestratorView, OrchestratorWorkspaceState};
use pubgen_core::{ProjectContent, WorkspaceDraft, export_workbook, open_pubgen, save_pubgen};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct OrchestratorProject {
    pub path: Option<PathBuf>,
    pub draft: Option<WorkspaceDraft>,
    pub status: String,
    pub dirty: bool,
    pub ui_synced: bool,
    pub text: Vec<(String, Vec<u8>)>,
    pub assets: Vec<(String, Vec<u8>)>,
}

impl OrchestratorProject {
    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        let opened = open_pubgen(path).map_err(|error| error.to_string())?;
        let csv = opened.content.csv;
        let text = opened.content.text.clone();
        let assets = opened.content.assets.clone();
        let metadata = opened.metadata;
        self.draft = Some(WorkspaceDraft {
            manifest: opened.manifest,
            metadata: metadata.clone(),
            glossary: opened.glossary,
            tm: opened.tm,
            csv,
            text: text.clone(),
            assets: assets.clone(),
            warnings: Vec::new(),
        });
        self.text = text;
        self.assets = assets;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        self.status = "Project opened".into();
        Ok(())
    }
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let draft = self.draft.as_ref().ok_or_else(|| "No workspace loaded".to_string())?;
        let text = if !draft.text.is_empty() { draft.text.clone() } else { self.text.clone() };
        let assets = if !draft.assets.is_empty() { draft.assets.clone() } else { self.assets.clone() };
        let content = ProjectContent { csv: draft.csv.clone(), text, assets, ..ProjectContent::default() };
        save_pubgen(path, &draft.manifest, &draft.metadata, &draft.glossary, &draft.tm, &content).map_err(|error| error.to_string())?;
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
            let text = if !draft.text.is_empty() { &draft.text } else { &self.text };
            let assets = if !draft.assets.is_empty() { &draft.assets } else { &self.assets };
            workspace::sync_external_workspace(
                ctx,
                self.path.as_ref().and_then(|path| path.to_str()).unwrap_or("Workbook"),
                &draft.metadata,
                &draft.tm,
                &draft.csv,
                text,
                assets,
            );
            self.ui_synced = true;
        }
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn title(&self) -> String {
        if let Some(draft) = &self.draft
            && !draft.metadata.title.trim().is_empty()
        {
            return draft.metadata.title.clone();
        }
        self.path.as_ref().and_then(|path| path.file_stem()).and_then(|stem| stem.to_str()).unwrap_or("Untitled publication").to_string()
    }
}

/// One Orchestrator workspace tab: its project plus the serialized 3-panel workspace state.
pub struct WorkspaceTab {
    pub project: OrchestratorProject,
    pub ui_state: Option<String>,
    pub dirty: bool,
}

/// Open Orchestrator workspaces, switched by tabs like DesignCraft document tabs.
#[derive(Default)]
pub struct OrchestratorTabs {
    pub tabs: Vec<WorkspaceTab>,
    pub active: usize,
    /// A dirty tab waiting for the close confirmation.
    pub pending_close: Option<usize>,
}

impl OrchestratorTabs {
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
    pub fn active_project(&self) -> Option<&OrchestratorProject> {
        self.tabs.get(self.active).map(|tab| &tab.project)
    }
    pub fn active_project_mut(&mut self) -> Option<&mut OrchestratorProject> {
        self.tabs.get_mut(self.active).map(|tab| &mut tab.project)
    }
    pub fn title(&self, index: usize) -> String {
        self.tabs.get(index).map(|tab| tab.project.title()).unwrap_or_else(|| "Untitled publication".into())
    }
    pub fn is_dirty(&self, index: usize) -> bool {
        self.tabs.get(index).is_some_and(|tab| tab.dirty)
    }
    /// Tab labels for the host-rendered strip: `(title, has unsaved changes)`.
    pub fn host_titles(&self) -> Vec<(String, bool)> {
        (0..self.tabs.len()).map(|index| (self.title(index), self.is_dirty(index))).collect()
    }

    /// Open a `.pubgen` package in a new tab (or activate the tab that already has it).
    pub fn open(&mut self, ctx: &egui::Context, path: &Path) -> Result<(), String> {
        if let Some(index) = self.index_of(path) {
            self.activate(ctx, index);
            return Ok(());
        }
        let mut project = OrchestratorProject::default();
        project.open(path)?;
        self.snapshot_active(ctx);
        self.tabs.push(WorkspaceTab { project, ui_state: None, dirty: false });
        self.active = self.tabs.len().saturating_sub(1);
        self.restore_active(ctx);
        Ok(())
    }

    /// Import an Excel workbook into a new tab.
    pub fn import(&mut self, ctx: &egui::Context, path: &Path) -> Result<(), String> {
        let report = pubgen_core::excel::ingest_workbook(path).map_err(|error| error.to_string())?;
        let name = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("publication").to_owned();
        let draft = WorkspaceDraft::from_ingest(name, report);
        self.snapshot_active(ctx);
        let text = draft.text.clone();
        let assets = draft.assets.clone();
        let project = OrchestratorProject {
            draft: Some(draft),
            path: Some(path.to_path_buf()),
            status: "Workbook imported".into(),
            dirty: true,
            text,
            assets,
            ..OrchestratorProject::default()
        };
        self.tabs.push(WorkspaceTab { project, ui_state: None, dirty: true });
        self.active = self.tabs.len().saturating_sub(1);
        self.restore_active(ctx);
        Ok(())
    }

    /// A blank studio tab for a new publication.
    pub fn new_tab(&mut self, ctx: &egui::Context) {
        self.snapshot_active(ctx);
        let state = OrchestratorWorkspaceState {
            view: OrchestratorView::Studio,
            cards: Vec::new(),
            content_files: Vec::new(),
            meta_title: "Untitled publication".into(),
            is_dirty: true,
            ..Default::default()
        };
        let ui_state = serde_json::to_string(&state).ok();
        self.tabs.push(WorkspaceTab { project: OrchestratorProject { dirty: true, ..OrchestratorProject::default() }, ui_state, dirty: true });
        self.active = self.tabs.len().saturating_sub(1);
        workspace::set_state(ctx, state);
    }

    pub fn activate(&mut self, ctx: &egui::Context, index: usize) {
        if index >= self.tabs.len() || index == self.active {
            return;
        }
        self.snapshot_active(ctx);
        self.active = index;
        self.restore_active(ctx);
    }

    /// Close a tab, asking first when its workspace has unsaved changes.
    pub fn request_close(&mut self, ctx: &egui::Context, index: usize) {
        if self.is_dirty(index) {
            self.pending_close = Some(index);
        } else {
            self.close(ctx, index);
        }
    }

    pub fn close(&mut self, ctx: &egui::Context, index: usize) {
        self.pending_close = None;
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active = 0;
            workspace::set_state(ctx, empty_state());
            return;
        }
        if index < self.active {
            self.active -= 1;
        } else if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        self.restore_active(ctx);
    }

    /// Save the visible 3-panel state into the active tab and adopt a package opened from the
    /// File menu or renamed by Save As.
    pub fn sync_from_ui(&mut self, ctx: &egui::Context) {
        let state = workspace::get_state(ctx);
        let Some(path) = state.recent_documents.first().filter(|path| path.to_ascii_lowercase().ends_with(".pubgen")) else {
            return;
        };
        if let Some(index) = self.index_of(Path::new(path)) {
            if index != self.active && !self.same_active_workspace(&state) {
                self.snapshot_active(ctx);
                self.active = index;
                self.restore_active(ctx);
            }
            return;
        }
        if self.same_active_workspace(&state) {
            if let Some(tab) = self.tabs.get_mut(self.active) {
                tab.project.path = Some(PathBuf::from(path));
            }
            return;
        }
        if !self.tabs.is_empty() {
            return;
        }
        let mut project = OrchestratorProject::default();
        if project.open(Path::new(path)).is_err() {
            return;
        }
        let ui_state = serde_json::to_string(&state).ok();
        self.tabs.push(WorkspaceTab { project, ui_state, dirty: state.is_dirty });
        self.active = self.tabs.len().saturating_sub(1);
    }

    /// Capture the visible workspace state and the active tab's dirty flag.
    pub fn publish_snapshot(&mut self, ctx: &egui::Context) {
        let state = workspace::get_state(ctx);
        let mut tab_state = state.clone();
        tab_state.view = OrchestratorView::Studio;
        let ui_state = serde_json::to_string(&tab_state).ok();
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.dirty = state.is_dirty;
            tab.ui_state = ui_state;
            if let Some(path) = state.recent_documents.first()
                && path.to_ascii_lowercase().ends_with(".pubgen")
            {
                tab.project.path = Some(PathBuf::from(path));
            }
        }
    }

    fn index_of(&self, path: &Path) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.project.path.as_deref() == Some(path))
    }

    fn restore_active(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.tabs.get(self.active) else {
            workspace::set_state(ctx, empty_state());
            return;
        };
        if let Some(json) = &tab.ui_state
            && let Ok(state) = serde_json::from_str::<OrchestratorWorkspaceState>(json)
        {
            workspace::set_state(ctx, state);
            return;
        }
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.project.sync_ui(ctx);
        }
    }

    /// True when the visible state is the active tab's workspace with only the package path or
    /// transient status fields changed (a Save As, not another document opened).
    fn same_active_workspace(&self, state: &OrchestratorWorkspaceState) -> bool {
        let Some(tab) = self.tabs.get(self.active) else { return false };
        let Some(json) = &tab.ui_state else { return false };
        let Ok(saved) = serde_json::from_str::<serde_json::Value>(json) else { return false };
        let Ok(current) = serde_json::to_value(state) else { return false };
        normalized(&saved) == normalized(&current)
    }

    fn snapshot_active(&mut self, ctx: &egui::Context) {
        if self.tabs.is_empty() {
            return;
        }
        let mut state = workspace::get_state(ctx);
        state.view = OrchestratorView::Studio;
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.dirty = state.is_dirty;
            tab.ui_state = serde_json::to_string(&state).ok();
        }
    }
}

fn empty_state() -> OrchestratorWorkspaceState {
    OrchestratorWorkspaceState {
        view: OrchestratorView::Home,
        cards: Vec::new(),
        content_files: Vec::new(),
        recent_documents: Vec::new(),
        ..Default::default()
    }
}

fn normalized(value: &serde_json::Value) -> serde_json::Value {
    let mut value = value.clone();
    if let Some(object) = value.as_object_mut() {
        for key in ["recent_documents", "execution_log", "status_message", "is_dirty"] {
            object.remove(key);
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_project_rejects_save() {
        let mut project = OrchestratorProject::default();
        assert!(project.save(Path::new("x.pubgen")).is_err());
    }
    #[test]
    fn tabs_start_empty() {
        let tabs = OrchestratorTabs::default();
        assert!(tabs.is_empty());
        assert!(tabs.active_project().is_none());
    }

    fn write_package(path: &Path, title: &str) {
        let manifest = pubgen_core::ProjectManifest { format_version: 1, name: title.into(), template: false, slots: Vec::new() };
        let metadata = pubgen_core::PublicationMetadata {
            profile_id: "bps_dda".into(),
            layout: pubgen_core::PageLayout::Single,
            language: pubgen_core::Language::Id,
            title: title.into(),
            title_en: None,
            cards: Vec::new(),
            extra: Default::default(),
        };
        let content = ProjectContent { csv: vec![("T1.csv".into(), b"a,b\n1,2\n".to_vec())], ..ProjectContent::default() };
        save_pubgen(path, &manifest, &metadata, &pubgen_core::Glossary::default(), &pubgen_core::TranslationMemory::default(), &content)
            .expect("package");
    }

    #[test]
    fn opening_packages_adds_tabs_and_keeps_each_workspace() {
        let dir = std::env::temp_dir().join(format!("pubgen-studio-tabs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let first = dir.join("first.pubgen");
        let second = dir.join("second.pubgen");
        write_package(&first, "First publication");
        write_package(&second, "Second publication");

        let ctx = egui::Context::default();
        let mut tabs = OrchestratorTabs::default();
        tabs.open(&ctx, &first).expect("open first");
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(workspace::get_state(&ctx).meta_title, "First publication");

        tabs.open(&ctx, &second).expect("open second");
        assert_eq!(tabs.tabs.len(), 2, "a second package opens a second tab");
        assert_eq!(tabs.active, 1);
        assert_eq!(workspace::get_state(&ctx).meta_title, "Second publication");

        // An edit in the second tab survives a round trip through the first tab.
        let mut state = workspace::get_state(&ctx);
        state.meta_title = "Second edited".into();
        workspace::set_state(&ctx, state);
        tabs.publish_snapshot(&ctx);

        tabs.activate(&ctx, 0);
        assert_eq!(workspace::get_state(&ctx).meta_title, "First publication");
        tabs.activate(&ctx, 1);
        assert_eq!(workspace::get_state(&ctx).meta_title, "Second edited");

        tabs.activate(&ctx, 0);
        tabs.close(&ctx, 0);
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(workspace::get_state(&ctx).meta_title, "Second edited");

        tabs.close(&ctx, 0);
        assert!(tabs.is_empty());
        assert_eq!(workspace::get_state(&ctx).view, OrchestratorView::Home);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_as_renames_the_active_tab_instead_of_opening_a_new_one() {
        let dir = std::env::temp_dir().join(format!("pubgen-studio-saveas-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("before.pubgen");
        let renamed = dir.join("after.pubgen");
        write_package(&path, "Save As publication");

        let ctx = egui::Context::default();
        let mut tabs = OrchestratorTabs::default();
        tabs.open(&ctx, &path).expect("open");
        tabs.publish_snapshot(&ctx);

        let mut state = workspace::get_state(&ctx);
        state.recent_documents = vec![renamed.to_string_lossy().to_string()];
        state.is_dirty = false;
        workspace::set_state(&ctx, state);
        tabs.sync_from_ui(&ctx);

        assert_eq!(tabs.tabs.len(), 1, "a renamed package stays one tab");
        assert_eq!(tabs.active_project().and_then(|project| project.path.as_deref()), Some(renamed.as_path()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_new_tab_does_not_replace_the_active_workspace() {
        let ctx = egui::Context::default();
        let mut tabs = OrchestratorTabs::default();
        tabs.new_tab(&ctx);
        assert_eq!(tabs.tabs.len(), 1);
        tabs.new_tab(&ctx);
        assert_eq!(tabs.tabs.len(), 2);
        assert_eq!(workspace::get_state(&ctx).view, OrchestratorView::Studio);
    }

    #[test]
    fn importing_xlsx_from_home_page_adds_a_new_tab_without_replacing_active_pubgen_tab() {
        let dir = std::env::temp_dir().join(format!("pubgen-studio-import-home-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let pkg_path = dir.join("active.pubgen");
        write_package(&pkg_path, "Existing Active Publication");

        let ctx = egui::Context::default();
        let mut tabs = OrchestratorTabs::default();
        // 1. User opens active .pubgen publication
        tabs.open(&ctx, &pkg_path).expect("open pubgen package");
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.active, 0);
        assert_eq!(tabs.title(0), "Existing Active Publication");

        // 2. User goes to Home Orchestrator Page
        let mut state = workspace::get_state(&ctx);
        state.view = OrchestratorView::Home;
        workspace::set_state(&ctx, state);
        tabs.publish_snapshot(&ctx);

        // 3. User imports manifest XLSX from Home Orchestrator Page
        let sample = Path::new("plan/pubgen/samples/brs/ntp202610.xlsx");
        if sample.is_file() {
            tabs.import(&ctx, sample).expect("import sample xlsx");

            // 4. Must load as a new worksheet tab (2 tabs total), NOT replace the active tab!
            assert_eq!(tabs.tabs.len(), 2, "XLSX import must create a new worksheet tab");
            assert_eq!(tabs.active, 1, "New imported tab should become active");
            assert_eq!(tabs.title(0), "Existing Active Publication", "First tab title must be preserved");
            assert_ne!(tabs.title(1), "Existing Active Publication", "Second tab must hold the imported workbook");
            assert_eq!(workspace::get_state(&ctx).view, OrchestratorView::Studio, "Imported tab must open into Studio view");

            // 5. Switching back to tab 0 restores the original publication
            tabs.activate(&ctx, 0);
            assert_eq!(tabs.active, 0);
            let restored = workspace::get_state(&ctx);
            assert_eq!(restored.meta_title, "Existing Active Publication");
            assert_eq!(restored.view, OrchestratorView::Studio);
        }

        std::fs::remove_dir_all(&dir).ok();
    }
}
