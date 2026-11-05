pub const APP_TITLE: &str = "Publication Orchestrator Studio";
pub const DEFAULT_WINDOW_SIZE: [f32; 2] = [1360.0, 860.0];
/// TCP port used by the DesignCraft control channel. Single source of truth.
pub const DC_PORT: u16 = 7979;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DcStatus {
    #[default]
    Off,
    Booting,
    Ok,
}

impl DcStatus {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            DcStatus::Off => "DesignCraft: off",
            DcStatus::Booting => "DesignCraft: booting…",
            DcStatus::Ok => "● DesignCraft: OK",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AppTheme {
    #[default]
    System,
    Light,
    Dark,
}

impl AppTheme {
    pub const ALL: [AppTheme; 3] = [AppTheme::System, AppTheme::Light, AppTheme::Dark];

    pub fn icon(self) -> &'static str {
        match self {
            Self::System => "💻",
            Self::Light => "☀️",
            Self::Dark => "🌙",
        }
    }

    pub fn label(self, lang: crate::i18n::UiLang) -> &'static str {
        match self {
            Self::System => crate::i18n::tr(lang, "theme_system"),
            Self::Light => crate::i18n::tr(lang, "theme_light"),
            Self::Dark => crate::i18n::tr(lang, "theme_dark"),
        }
    }
}

/// Main application view mode: Welcome/Home page vs 3-Column Studio Workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AppView {
    #[default]
    Home,
    Studio,
}

/// Center panel tab selector below the action buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CenterTab {
    #[default]
    Content,
    Preview,
    Log,
}

/// Preview mode inside CenterTab::Preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum PreviewMode {
    #[default]
    Cover,
    Chart,
    Spreads,
}

pub struct App {
    pub view: AppView,
    pub preview_mode: PreviewMode,
    pub recent_projects: Vec<std::path::PathBuf>,
    pub project_path: Option<std::path::PathBuf>,
    pub source_excel_path: Option<std::path::PathBuf>,
    pub log: Vec<String>,
    pub dc: Option<crate::process::DesignCraftProcess>,
    pub dc_status: DcStatus,
    pub client: Option<crate::control_client::ControlClient>,
    pub last_health: std::time::Instant,
    pub project: crate::project::Project,
    pub last_output: Option<std::path::PathBuf>,
    pub last_output_doc: Option<std::path::PathBuf>,
    pub audit_report: Option<crate::audit::AuditReport>,
    pub llm_base: String,
    pub llm_model: String,
    pub llm_key: String,
    pub glossary_term: String,
    pub glossary_replacement: String,
    pub glossary_dnt: bool,
    pub ui_lang: crate::i18n::UiLang,
    pub theme: AppTheme,
    pub selected_profile: String,
    pub cover_style: crate::covers::CoverStyle,
    pub ai_cover_theme: String,
    pub toc_entries: Vec<crate::toc::TocEntry>,
    pub chart_type: crate::vector::ChartType,
    pub chart_title: String,
    pub center_tab: CenterTab,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            view: AppView::Home,
            preview_mode: PreviewMode::Cover,
            recent_projects: Vec::new(),
            project_path: None,
            source_excel_path: None,
            log: Vec::new(),
            dc: None,
            dc_status: DcStatus::Off,
            client: None,
            last_health: std::time::Instant::now(),
            project: crate::project::Project::new("Untitled"),
            last_output: None,
            last_output_doc: None,
            audit_report: None,
            llm_base: std::env::var("PUBGEN_LLM_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
            llm_model: std::env::var("PUBGEN_LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into()),
            llm_key: std::env::var("PUBGEN_LLM_KEY").unwrap_or_default(),
            glossary_term: String::new(),
            glossary_replacement: String::new(),
            glossary_dnt: false,
            ui_lang: crate::i18n::UiLang::Id,
            theme: AppTheme::System,
            selected_profile: "bps_dda".to_string(),
            cover_style: crate::covers::CoverStyle::Horizon,
            ai_cover_theme: "pertanian".to_string(),
            toc_entries: Vec::new(),
            chart_type: crate::vector::ChartType::BarClustered,
            chart_title: "Grafik Statistik".to_string(),
            center_tab: CenterTab::Content,
        };
        app.load_recent_projects();
        app
    }

    pub fn recent_projects_file() -> std::path::PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let dir = std::path::PathBuf::from(appdata).join("pubgen");
            let _ = std::fs::create_dir_all(&dir);
            dir.join("recent_projects.json")
        } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let dir = std::path::PathBuf::from(home).join(".pubgen");
            let _ = std::fs::create_dir_all(&dir);
            dir.join("recent_projects.json")
        } else {
            std::path::PathBuf::from("recent_projects.json")
        }
    }

    pub fn load_recent_projects(&mut self) {
        let path = Self::recent_projects_file();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(list) = serde_json::from_str::<Vec<std::path::PathBuf>>(&data) {
                self.recent_projects = list.into_iter().filter(|p| p.exists()).collect();
            }
        }
    }

    pub fn save_recent_projects(&self) {
        let path = Self::recent_projects_file();
        if let Ok(data) = serde_json::to_string_pretty(&self.recent_projects) {
            let _ = std::fs::write(&path, data);
        }
    }

    pub fn add_recent_project(&mut self, path: std::path::PathBuf) {
        self.recent_projects.retain(|p| p != &path);
        self.recent_projects.insert(0, path);
        if self.recent_projects.len() > 10 {
            self.recent_projects.truncate(10);
        }
        self.save_recent_projects();
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 800 {
            let overflow = self.log.len() - 800;
            self.log.drain(0..overflow);
        }
    }

    pub fn set_publication_profile(&mut self, profile_id: &str) {
        self.selected_profile = profile_id.to_string();
        self.project.metadata.apply_profile(profile_id);
        self.log(format!("switched publication profile to: {profile_id}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_theme_properties() {
        assert_eq!(AppTheme::default(), AppTheme::System);
        assert_eq!(AppTheme::ALL.len(), 3);
        for t in AppTheme::ALL {
            assert!(!t.icon().is_empty());
            assert!(!t.label(crate::i18n::UiLang::Id).is_empty());
            assert!(!t.label(crate::i18n::UiLang::En).is_empty());
        }
    }

    #[test]
    fn test_app_view_and_center_tabs() {
        let app = App::new();
        assert_eq!(app.view, AppView::Home);
        assert_eq!(app.center_tab, CenterTab::Content);
        assert_eq!(app.preview_mode, PreviewMode::Cover);
    }

    #[test]
    fn test_recent_projects_add_and_dedup() {
        let mut app = App::new();
        app.recent_projects.clear();
        let p1 = std::path::PathBuf::from("proj1.pubgen");
        let p2 = std::path::PathBuf::from("proj2.pubgen");

        app.add_recent_project(p1.clone());
        assert_eq!(app.recent_projects.len(), 1);
        assert_eq!(app.recent_projects[0], p1);

        app.add_recent_project(p2.clone());
        assert_eq!(app.recent_projects.len(), 2);
        assert_eq!(app.recent_projects[0], p2);
        assert_eq!(app.recent_projects[1], p1);

        // Re-adding p1 moves it to index 0 with no duplication
        app.add_recent_project(p1.clone());
        assert_eq!(app.recent_projects.len(), 2);
        assert_eq!(app.recent_projects[0], p1);
        assert_eq!(app.recent_projects[1], p2);
    }
}
