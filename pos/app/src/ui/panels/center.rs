// Center panel: header/footer preview, slot fit, QA checklist, actions, tabs.
// Port of panels::center. Content tab metadata cards = Phase 10; Preview = Phase 9.

use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Length, Theme,
};

use crate::i18n::tr;
use crate::main_ui::CenterTab;

use super::super::message::*;
use super::super::theme;
use super::super::UiApp;
use super::left::{rule, section, text_btn, text_btn_maybe, weak};

pub fn center(ui: &UiApp) -> Element<'_, Message> {
    let app = &ui.app;
    let lang = app.ui_lang;
    let dark = theme::is_dark(app.theme, ui.system_dark);

    let profile = crate::profiles::ProfileManager::get_profile(&app.selected_profile);
    let g = &profile.geometry;

    // 1. Live header/footer preview card
    let (label, l1, l2, label_color) = if profile.id == "bps_brs" {
        let (h1, h2) = crate::header_footer::resolve_brs_header_texts(
            &app.project.metadata.title,
            &app.project.metadata.release_no,
            &app.project.metadata.release_date,
            &app.project.metadata.release_month,
        );
        (
            "RUNNING HEADER:",
            h1,
            h2,
            iced::Color::from_rgb8(230, 81, 0),
        )
    } else {
        let (f1, f2) = crate::header_footer::resolve_dda_footer_lines(
            &app.project.metadata.title,
            &app.project.metadata.subtitle,
            &app.project.metadata.title_en,
            &app.project.metadata.date,
            &app.project.metadata.release_date,
            &app.project.metadata.release_month,
        );
        (
            "RUNNING FOOTER:",
            f1,
            f2,
            iced::Color::from_rgb8(25, 118, 210),
        )
    };

    let preview_card = container(
        column![
            row![
                text(label).size(11).color(label_color),
                text(l1).size(11),
            ]
            .spacing(6),
            text(format!("    {l2}")).size(11),
        ]
        .spacing(2),
    )
    .width(Length::Fill)
    .padding(8)
    .style(move |_theme: &Theme| container::Style {
        background: Some(if dark {
            theme::POPOVER_BG_DARK
        } else {
            iced::Color::from_rgb8(250, 250, 252)
        }.into()),
        border: iced::Border {
            width: 1.0,
            color: if dark {
                iced::Color::from_rgb8(55, 60, 72)
            } else {
                iced::Color::from_rgb8(210, 210, 210)
            },
            radius: (4.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let hf_open = if ui.touched_panels.contains("center_hf_preview") {
        ui.open_panels.contains("center_hf_preview")
    } else {
        true
    };

    let hf_content = column![
        weak(
            format!(
                "Facing: {} | Margins: T:{:.1} B:{:.1} L:{:.1} R:{:.1} mm",
                if g.facing_pages { tr(lang, "yes") } else { tr(lang, "no") },
                g.margins_mm.top,
                g.margins_mm.bottom,
                g.margins_mm.inside,
                g.margins_mm.outside,
            ),
            dark,
        ),
        rule(),
        preview_card,
    ]
    .spacing(6);

    let hf_title = format!("{} — {}", tr(lang, "header_footer_preview"), profile.name);
    let hf_group = container(super::super::widgets::collapsible(
        &hf_title,
        hf_open,
        Message::PanelToggled("center_hf_preview"),
        hf_content,
    ))
    .width(Length::Fill)
    .padding(8)
    .style(move |theme: &Theme| container::Style {
        border: iced::Border {
            radius: (4.0).into(),
            width: 1.0,
            color: theme.palette().text.scale_alpha(0.2),
        },
        ..Default::default()
    });

    // 2. Slot character fit
    let slots_with_budget: Vec<_> = app
        .project
        .slots
        .iter()
        .filter(|s| s.budget.is_some() || !s.text.is_empty())
        .collect();

    let mut body = column![hf_group].spacing(8);

    if !slots_with_budget.is_empty() {
        let slots_open = if ui.touched_panels.contains("center_slots_fit") {
            ui.open_panels.contains("center_slots_fit")
        } else {
            true
        };

        let mut rows = column![
            row![
                text(tr(lang, "slot_col_slot")).size(12),
                text(tr(lang, "char_budget")).size(12).width(90),
                text(tr(lang, "slot_col_status")).size(12).width(70),
                text(tr(lang, "slot_col_summary")).size(12),
            ]
            .spacing(8)
        ]
        .spacing(2);

        for slot in &slots_with_budget {
            let count = slot.text.chars().count();
            let status: iced::widget::text::Text<'static> = match slot.budget {
                Some(b) if count <= b as usize => text(tr(lang, "status_ok").to_string())
                    .size(11)
                    .color(iced::Color::from_rgb8(46, 125, 50)),
                Some(_) => text(tr(lang, "status_over").to_string())
                    .size(11)
                    .color(iced::Color::from_rgb8(198, 40, 40)),
                None => text("—").size(11),
            };
            let budget_label = match slot.budget {
                Some(b) => format!("{count}/{b}"),
                None => format!("{count}"),
            };
            let preview: String = slot.text.chars().take(28).collect();
            rows = rows.push(
                row![
                    text(&slot.name).size(11).width(140),
                    text(budget_label).size(11).width(90),
                    status.width(Length::Fixed(70.0)),
                    text(format!("{preview}…")).size(11),
                ]
                .spacing(8),
            );
        }

        let slots_title = format!(
            "{} ({} {})",
            tr(lang, "slot_character_fit"),
            slots_with_budget.len(),
            tr(lang, "slot_col_slot")
        );
        let slots_collapsible = super::super::widgets::collapsible(
            &slots_title,
            slots_open,
            Message::PanelToggled("center_slots_fit"),
            rows,
        );

        body = body.push(
            container(slots_collapsible)
                .width(Length::Fill)
                .padding(8)
                .style(move |theme: &Theme| container::Style {
                    border: iced::Border {
                        radius: (4.0).into(),
                        width: 1.0,
                        color: theme.palette().text.scale_alpha(0.2),
                    },
                    ..Default::default()
                }),
        );
    }

    // 3. Preflight QA checklist
    let mut qa_row = row![
        text(tr(lang, "qa_checklist")).size(13),
        text_btn(tr(lang, "run_audit").to_string(), "", Message::Action("pub_qa_audit")),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    if let Some(rep) = &app.audit_report {
        if rep.is_clean() {
            qa_row = qa_row.push(
                text(format!("● {}", tr(lang, "audit_clean")))
                    .size(12)
                    .color(iced::Color::from_rgb8(46, 125, 50)),
            );
        } else {
            qa_row = qa_row.push(
                text(format!(
                    "🔴 {} {} | 🟡 {} {}",
                    rep.errors,
                    tr(lang, "errors"),
                    rep.warnings,
                    tr(lang, "warnings")
                ))
                .size(12),
            );
        }
    }

    let mut qa_group = column![qa_row].spacing(4);
    if let Some(rep) = &app.audit_report {
        if !rep.issues.is_empty() {
            qa_group = qa_group.push(rule());
            let mut issues = column![].spacing(2);
            for issue in &rep.issues {
                let icon = if issue.severity.eq_ignore_ascii_case("error") {
                    "🔴"
                } else if issue.severity.eq_ignore_ascii_case("warning") {
                    "🟡"
                } else {
                    "ℹ"
                };
                let mut line = row![
                    text(icon).size(11),
                    text(&issue.kind).size(11),
                    text(&issue.message).size(11),
                ]
                .spacing(6);
                if let Some(p) = issue.page {
                    line = line.push(weak(format!("{}: {p}", tr(lang, "page_prefix")), dark));
                }
                issues = issues.push(line);
            }
            qa_group = qa_group.push(
                scrollable(issues)
                    .height(100)
                    .width(Length::Fill),
            );
        }
    }

    body = body.push(
        container(qa_group).width(Length::Fill).padding(8).style(|theme: &Theme| {
            container::Style {
                border: iced::Border {
                    radius: (4.0).into(),
                    width: 1.0,
                    color: theme.palette().text.scale_alpha(0.2),
                },
                ..Default::default()
            }
        }),
    );

    // 4. Action buttons + tab bar + tab content
    let actions = row![text_btn_maybe(
        format!("👁 {}", tr(lang, "view_last_pdf")),
        if app.last_output.is_some() {
            tr(lang, "tip_open_last_pdf")
        } else {
            tr(lang, "no_output_yet")
        },
        app.last_output
            .as_ref()
            .map(|_| Message::Action("file_open_pdf")),
    )]
    .spacing(8);

    let tab = |label: String, target: CenterTab| -> Element<'static, Message> {
        let selected = app.center_tab == target;
        button(text(label).size(12))
            .padding([4, 10])
            .on_press(Message::TabSelected(target))
            .style(move |theme: &Theme, _status| {
                let (bg, border_w) = if selected {
                    (theme.palette().text.scale_alpha(0.12), 1.0)
                } else {
                    (iced::Color::TRANSPARENT, 0.0)
                };
                button::Style {
                    background: Some(bg.into()),
                    text_color: theme.palette().text,
                    border: iced::Border {
                        radius: (4.0).into(),
                        width: border_w,
                        color: theme.palette().text.scale_alpha(0.3),
                    },
                    ..Default::default()
                }
            })
            .into()
    };

    let tab_bar = row![
        tab(format!("🗂 {}", tr(lang, "tab_page_structure")), CenterTab::Content),
        tab("🎨 Visual Live Preview".into(), CenterTab::Preview),
        tab(format!("📋 {}", tr(lang, "run_log")), CenterTab::Log),
    ]
    .spacing(4);

    let tab_content: Element<'_, Message> = match app.center_tab {
        CenterTab::Content => super::super::metadata::publication_cards(ui),
        CenterTab::Preview => super::super::preview::preview(ui),
        CenterTab::Log => {
            let mut lines = column![].spacing(0);
            if app.log.is_empty() {
                lines = lines.push(weak(tr(lang, "status_idle"), dark));
            }
            for line in &app.log {
                lines = lines.push(text(line.clone()).size(11));
            }
            scrollable(lines)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    };

    column![
        section(tr(lang, "run_and_verification")),
        body,
        actions,
        rule(),
        tab_bar,
        rule(),
        tab_content,
    ]
    .spacing(6)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
