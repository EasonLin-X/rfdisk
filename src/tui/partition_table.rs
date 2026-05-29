use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::{
    app::state::{App, Focus, PartitionRow},
    model::{DraftPartition, PendingState},
    util::{
        i18n::{tr, Msg},
        size::format_size,
    },
};

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let Some(disk) = app.selected_disk() else {
        let empty = Paragraph::new(tr(app.lang, Msg::NoSelectedDisk))
            .block(
                Block::default()
                    .title(tr(app.lang, Msg::PartitionTable))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
        return;
    };

    let partition_type_hint = disk
        .editable_table_type()
        .map(|table_type| table_type.as_str())
        .unwrap_or_else(|| disk.table_label.as_str());

    let rows: Vec<Row> = app
        .partition_rows()
        .into_iter()
        .map(|row| match row {
            PartitionRow::Partition {
                partition: part, ..
            } => {
                let state = match part.pending {
                    PendingState::Existing => "",
                    PendingState::New => "new",
                    PendingState::Deleted => "deleted",
                    PendingState::Modified => "modified",
                };

                Row::new(vec![
                    Cell::from(part.display_name.clone()),
                    Cell::from(part.start_sector.to_string()),
                    Cell::from(format_size(part.size_bytes)),
                    Cell::from(display_part_type(&part)),
                    Cell::from(display_fs_type(&part)),
                    Cell::from(display_mount(&part)),
                    Cell::from(state),
                ])
            }
            PartitionRow::FreeSpace(segment) => Row::new(vec![
                Cell::from(tr(app.lang, Msg::FreeSpace)),
                Cell::from(segment.start_sector.to_string()),
                Cell::from(format_size(segment.size_bytes())),
                Cell::from(tr(app.lang, Msg::Unallocated)),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from(""),
            ])
            .style(Style::default().fg(Color::DarkGray)),
        })
        .collect();

    let dirty = if app.has_active_draft_changes() {
        " pending"
    } else {
        ""
    };
    let title = format!(
        " Partition Table: {} ({}, {}{}) ",
        disk.dev_path,
        format_size(disk.size_bytes),
        partition_type_hint,
        dirty
    );
    let border_color = if app.focus == Focus::Partitions {
        Color::Cyan
    } else {
        Color::White
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(13),
            Constraint::Length(13),
            Constraint::Length(22),
            Constraint::Length(10),
            Constraint::Min(12),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec![
            tr(app.lang, Msg::Partition),
            tr(app.lang, Msg::Start),
            tr(app.lang, Msg::DiskSize),
            tr(app.lang, Msg::PartType),
            tr(app.lang, Msg::Fs),
            tr(app.lang, Msg::Mount),
            tr(app.lang, Msg::Draft),
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
    .row_highlight_style(Style::default().fg(Color::Black).bg(Color::LightBlue))
    .highlight_symbol(" > ");

    frame.render_stateful_widget(table, area, &mut app.partition_table_state);
}

fn display_part_type(part: &DraftPartition) -> String {
    display_or_dash(&part.partition_type)
}

fn display_fs_type(part: &DraftPartition) -> String {
    if part.is_swap {
        return "swap".to_string();
    }

    display_or_dash(&part.fs_type)
}

fn display_mount(part: &DraftPartition) -> String {
    if part.is_swap {
        return "[swap]".to_string();
    }

    if part.mount_points.is_empty() {
        "-".to_string()
    } else {
        part.mount_points.join(",")
    }
}

fn display_or_dash(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "-".to_string()
    } else {
        trimmed.to_string()
    }
}
