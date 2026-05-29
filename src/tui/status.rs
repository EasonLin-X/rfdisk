use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::state::{App, InputMode},
    model::disk::ScanSource,
    util::{
        i18n::{tr, Msg},
        sector::bytes_to_sectors,
        size::format_size,
    },
};

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    let disk_text = app
        .selected_disk()
        .map(|disk| {
            format!(
                "{}{} {}",
                tr(app.lang, Msg::Selected),
                disk.dev_path,
                if disk.is_protected() {
                    disk.guard.label()
                } else {
                    ""
                }
            )
        })
        .unwrap_or_else(|| tr(app.lang, Msg::SelectedNone).to_string());

    let mut first_line = vec![
        Span::styled(&app.status, Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(disk_text, Style::default().fg(Color::Gray)),
    ];

    if let Some(disk) = app.selected_disk() {
        let scan_color = match disk.scan_status.source {
            ScanSource::Sfdisk => Color::Green,
            ScanSource::SysfsFallback if disk.scan_status.permission_limited => Color::Red,
            ScanSource::SysfsFallback => Color::Yellow,
        };
        first_line.push(Span::raw(tr(app.lang, Msg::Scan)));
        first_line.push(Span::styled(
            disk.scan_status.label(),
            Style::default().fg(scan_color),
        ));
    }

    let mut lines = vec![Line::from(first_line)];

    if let Some(disk) = app.selected_disk() {
        if disk.scan_status.is_degraded() {
            let prefix = if disk.scan_status.permission_limited {
                tr(app.lang, Msg::PermissionLimited)
            } else {
                tr(app.lang, Msg::ScanFallback)
            };
            let mut spans = vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::raw(tr(app.lang, Msg::ScanFallbackDetail)),
            ];

            if let Some(reason) = disk.scan_status.short_reason() {
                spans.push(Span::raw(tr(app.lang, Msg::Reason)));
                spans.push(Span::styled(reason, Style::default().fg(Color::DarkGray)));
            }
            lines.push(Line::from(spans));
        }
    }

    if app.has_committed_drafts() {
        lines.push(Line::from(vec![
            Span::raw(tr(app.lang, Msg::CommittedDraftsPrefix)),
            Span::styled(
                tr(app.lang, Msg::CommittedDraftsWrite),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(tr(app.lang, Msg::CommittedDraftsSuffix)),
        ]));
    }

    match app.input_mode {
        InputMode::Size => {
            if let Some(adjustment) = app.pending_size_adjustment {
                lines.push(Line::from(vec![
                    Span::raw(tr(app.lang, Msg::SizeLabel)),
                    Span::styled(&app.input, Style::default().fg(Color::Yellow)),
                    Span::raw("  "),
                    Span::styled(
                        format!("~ {}", format_size(adjustment.requested_bytes)),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(format!(
                        "  actual: {} sectors",
                        bytes_to_sectors(adjustment.adjusted_bytes)
                    )),
                ]));
            } else if app.input_is_default {
                lines.push(Line::from(vec![
                    Span::raw("Default size: "),
                    Span::styled(&app.input, Style::default().fg(Color::DarkGray)),
                    Span::raw("  (type to replace it, Enter to accept)"),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw(tr(app.lang, Msg::SizeLabel)),
                    Span::styled(&app.input, Style::default().fg(Color::Yellow)),
                ]));
            }
        }
        InputMode::TypePicker => lines.push(Line::from(vec![
            Span::raw(tr(app.lang, Msg::TypePickerStatusPrefix)),
            Span::styled(
                tr(app.lang, Msg::TypePickerStatusPart),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(tr(app.lang, Msg::TypePickerStatusMiddle)),
            Span::styled(
                tr(app.lang, Msg::TypePickerStatusTable),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(tr(app.lang, Msg::TypePickerStatusSuffix)),
        ])),
        InputMode::SetPlan => lines.push(Line::from(vec![
            Span::raw(tr(app.lang, Msg::SetPlanStatusPrefix)),
            Span::styled(
                tr(app.lang, Msg::SetPlanUnderDevelopment),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(tr(app.lang, Msg::SetPlanStatusSuffix)),
        ])),
        InputMode::WriteConfirm => lines.push(Line::from(vec![
            Span::raw(tr(app.lang, Msg::ConfirmLabel)),
            Span::styled(&app.input, Style::default().fg(Color::Yellow)),
            Span::raw(tr(app.lang, Msg::WriteConfirmHint)),
        ])),
        InputMode::Normal => {}
    }

    let status = Paragraph::new(lines)
        .block(
            Block::default()
                .title(tr(app.lang, Msg::Status))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White));
    frame.render_widget(status, area);
}
