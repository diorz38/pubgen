// Bottom status bar: a four-segment publish-readiness indicator.
//
// Every segment is derived from project state, so the bar answers "can I
// publish this?" at a glance:
//   Sources | Pages | QA Fit | Status
// Colors follow the same rule everywhere: gray = nothing yet, amber = partial,
// green = ready.

use iced::{
    widget::{container, row, text},
    Element, Length, Theme,
};

use crate::i18n::{tr, UiLang};
use crate::main_ui::App;

use super::super::theme;
use super::super::Message;
use super::super::UiApp;

/// Readiness of one segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    /// Nothing to report yet (gray).
    Empty,
    /// Some progress, not publish-ready (amber).
    Partial,
    /// Fully ready (green).
    Ready,
}

/// State projected from the project, kept independent of iced for tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub sources: usize,
    pub slots: usize,
    pub fitted: usize,
    pub bound: usize,
    pub total_pages: usize,
    /// Output file name (never a full directory path).
    pub last_output: Option<String>,
}

impl Summary {
    pub fn from_app(app: &App) -> Self {
        let slots = app.project.slots.len();
        let fitted = app
            .project
            .slots
            .iter()
            .filter(|slot| !slot.text.is_empty())
            .count();
        let bound = app
            .project
            .metadata
            .pages
            .iter()
            .filter(|page| app.project.page_source(page).is_some())
            .count();
        let last_output = app.last_output.as_ref().map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string())
        });

        Self {
            sources: app.project.content.len(),
            slots,
            fitted,
            bound,
            total_pages: app.project.metadata.pages.len(),
            last_output,
        }
    }

    pub fn sources_state(&self) -> Readiness {
        if self.sources > 0 {
            Readiness::Ready
        } else {
            Readiness::Empty
        }
    }

    pub fn pages_state(&self) -> Readiness {
        if self.total_pages == 0 {
            Readiness::Empty
        } else if self.bound == self.total_pages {
            Readiness::Ready
        } else {
            Readiness::Partial
        }
    }

    /// Slot fill percentage, rounded to the nearest whole point.
    pub fn fit_pct(&self) -> u32 {
        if self.slots == 0 {
            0
        } else {
            ((self.fitted as f32 / self.slots as f32) * 100.0).round() as u32
        }
    }

    pub fn fit_state(&self) -> Readiness {
        if self.slots == 0 {
            Readiness::Empty
        } else if self.fitted == self.slots {
            Readiness::Ready
        } else {
            Readiness::Partial
        }
    }

    pub fn status_state(&self) -> Readiness {
        if self.last_output.is_some() {
            Readiness::Ready
        } else if self.sources > 0 || self.bound > 0 {
            Readiness::Partial
        } else {
            Readiness::Empty
        }
    }

    pub fn status_text(&self, lang: UiLang) -> String {
        match (&self.last_output, self.status_state()) {
            (Some(name), _) => format!(
                "{}: {} · {}",
                tr(lang, "status_label"),
                tr(lang, "status_ready"),
                middle_ellipsis(name, 60)
            ),
            (None, Readiness::Partial) => format!(
                "{}: {}",
                tr(lang, "status_label"),
                tr(lang, "status_not_compiled")
            ),
            (None, _) => format!(
                "{}: {}",
                tr(lang, "status_label"),
                tr(lang, "status_empty")
            ),
        }
    }
}

/// Shortens `value` to `max` characters by eliding the middle with `…`.
pub fn middle_ellipsis(value: &str, max: usize) -> String {
    let count = value.chars().count();
    if count <= max || max == 0 {
        return value.to_string();
    }

    let keep = max.saturating_sub(1);
    let head_len = keep * 2 / 3;
    let tail_len = keep - head_len;

    let head: String = value.chars().take(head_len).collect();
    let mut tail: Vec<char> = value.chars().rev().take(tail_len).collect();
    tail.reverse();
    let tail: String = tail.into_iter().collect();

    format!("{head}…{tail}")
}

fn state_color(state: Readiness, dark: bool) -> iced::Color {
    match state {
        Readiness::Ready => theme::success(dark),
        Readiness::Partial => theme::warning(dark),
        Readiness::Empty => theme::muted(dark),
    }
}

fn segment(value: String, state: Readiness, dark: bool) -> Element<'static, Message> {
    text(value).size(11).color(state_color(state, dark)).into()
}

fn separator(dark: bool) -> Element<'static, Message> {
    text("·").size(11).color(theme::muted(dark)).into()
}

pub fn bottom_bar(ui: &UiApp) -> Element<'_, Message> {
    let app = &ui.app;
    let lang = app.ui_lang;
    let dark = theme::is_dark(app.theme, ui.system_dark);
    let summary = Summary::from_app(app);

    let sources = format!(
        "{}: {} {}",
        tr(lang, "status_sources"),
        summary.sources,
        tr(lang, "status_files_unit")
    );
    let pages = format!(
        "{}: {}/{}",
        tr(lang, "pages"),
        summary.bound,
        summary.total_pages
    );
    let fit = format!("{}: {}%", tr(lang, "status_qa_fit"), summary.fit_pct());
    let (status, status_state) = if ui.busy {
        (
            format!("⏳ {}", tr(lang, "status_busy")),
            Readiness::Partial,
        )
    } else {
        (summary.status_text(lang), summary.status_state())
    };

    let bar = row![
        segment(sources, summary.sources_state(), dark),
        separator(dark),
        segment(pages, summary.pages_state(), dark),
        separator(dark),
        segment(fit, summary.fit_state(), dark),
        separator(dark),
        segment(status, status_state, dark),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let bar = super::super::widgets::tooltip(bar, tr(lang, "bottom_legend"));

    container(bar)
        .width(Length::Fill)
        .padding([4, 10])
        .style(move |theme: &Theme| container::Style {
            background: Some(theme.palette().background.scale_alpha(1.0).into()),
            border: iced::Border {
                width: 0.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(sources: usize, slots: usize, fitted: usize, bound: usize, total: usize) -> Summary {
        Summary {
            sources,
            slots,
            fitted,
            bound,
            total_pages: total,
            last_output: None,
        }
    }

    #[test]
    fn empty_project_is_all_gray() {
        let empty = summary(0, 0, 0, 0, 0);
        assert_eq!(empty.sources_state(), Readiness::Empty);
        assert_eq!(empty.pages_state(), Readiness::Empty);
        assert_eq!(empty.fit_pct(), 0);
        assert_eq!(empty.fit_state(), Readiness::Empty);
        assert_eq!(empty.status_state(), Readiness::Empty);
    }

    #[test]
    fn pages_are_ready_only_when_all_bound() {
        let partial = summary(2, 0, 0, 1, 3);
        assert_eq!(partial.pages_state(), Readiness::Partial);

        let ready = summary(2, 0, 0, 3, 3);
        assert_eq!(ready.pages_state(), Readiness::Ready);
    }

    #[test]
    fn fit_percentage_uses_all_slots() {
        assert_eq!(summary(1, 4, 1, 0, 0).fit_pct(), 25);
        assert_eq!(summary(1, 4, 4, 0, 0).fit_pct(), 100);
        assert_eq!(summary(1, 4, 4, 0, 0).fit_state(), Readiness::Ready);
        assert_eq!(summary(1, 4, 1, 0, 0).fit_state(), Readiness::Partial);
    }

    #[test]
    fn status_is_amber_without_output_and_green_with_output() {
        let sourced = summary(2, 0, 0, 3, 3);
        assert_eq!(sourced.status_state(), Readiness::Partial);

        let mut compiled = sourced;
        compiled.last_output = Some("annual-report.pdf".into());
        assert_eq!(compiled.status_state(), Readiness::Ready);
        assert!(compiled.status_text(UiLang::En).contains("annual-report.pdf"));
    }

    #[test]
    fn long_output_names_are_middle_elided() {
        let short = "report.pdf";
        assert_eq!(middle_ellipsis(short, 60), short);

        let long = "a".repeat(40) + &"b".repeat(40);
        let shortened = middle_ellipsis(&long, 60);
        assert_eq!(shortened.chars().count(), 60);
        assert!(shortened.contains('…'));
        assert!(shortened.starts_with('a'));
        assert!(shortened.ends_with('b'));
    }
}
