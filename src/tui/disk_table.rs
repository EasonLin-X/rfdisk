use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::{
    app::state::{App, Focus},
    util::{
        i18n::{tr, Msg},
        size::format_size,
    },
};

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.disks.is_empty() {
        let empty = Paragraph::new(tr(app.lang, Msg::NoDiskDetected))
            .block(
                Block::default()
                    .title(tr(app.lang, Msg::PhysicalDisks))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::Red));
        frame.render_widget(empty, area);
        return;
    }

    let rows = app.disks.iter().map(|disk| {
        let mut row = Row::new(vec![
            Cell::from(disk.name.clone()),
            Cell::from(disk.guard.label()),
            Cell::from(disk.dev_path.clone()),
            Cell::from(format_size(disk.size_bytes)),
            Cell::from(disk.table_label.as_str()),
            Cell::from(disk.kind.as_str()),
            Cell::from(disk.model.clone()),
            Cell::from(disk.serial.clone()),
        ]);
        if app.drafts.contains_key(&disk.stable_id()) {
            row = row.style(Style::default().bg(Color::DarkGray).fg(Color::Yellow));
        }
        row
    });

    let title = match app.focus {
        Focus::Disks => tr(app.lang, Msg::PhysicalDisksFocused),
        Focus::Partitions => tr(app.lang, Msg::PhysicalDisksTab),
    };
    let border_color = if app.focus == Focus::Disks {
        Color::Cyan
    } else {
        Color::Green
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(24),
        ],
    )
    .header(
        Row::new(vec![
            tr(app.lang, Msg::DiskName),
            tr(app.lang, Msg::DiskGuard),
            tr(app.lang, Msg::DiskPath),
            tr(app.lang, Msg::DiskSize),
            tr(app.lang, Msg::DiskTable),
            tr(app.lang, Msg::DiskKind),
            tr(app.lang, Msg::DiskModel),
            tr(app.lang, Msg::DiskSerial),
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    )
    .row_highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
    .highlight_symbol(" > ");

    frame.render_stateful_widget(table, area, &mut app.disk_table_state);
}
