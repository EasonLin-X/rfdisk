use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::{
    app::state::{App, TypePickerColumn},
    model::{PartitionTableType, COMMON_PARTITION_TYPES},
    util::i18n::{tr, Msg},
};

pub(crate) fn render_type_picker(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(area, 78, 72);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(tr(app.lang, Msg::TypePickerTitle))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, popup);

    let inner = popup.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(inner);

    render_part_types(frame, columns[0], app);
    render_table_types(frame, columns[1], app);
}

fn render_part_types(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.type_picker_column == TypePickerColumn::PartType;
    let table_type = app.active_table_type();
    let items = COMMON_PARTITION_TYPES
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let selected = focused && index == app.type_picker_part_idx;
            let supported = choice.is_supported(table_type);
            let label = if supported {
                choice.name.to_string()
            } else {
                format!("{}  {}", choice.name, tr(app.lang, Msg::Disabled))
            };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if !supported {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(label).style(style)
        })
        .collect::<Vec<_>>();

    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let list = List::new(items)
        .block(
            Block::default()
                .title(tr(app.lang, Msg::PartType))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border)),
        )
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(list, area);
}

fn render_table_types(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.type_picker_column == TypePickerColumn::TableType;
    let active = app.active_table_type();
    let items = PartitionTableType::ALL
        .iter()
        .enumerate()
        .map(|(index, table_type)| {
            let selected = focused && index == app.type_picker_table_idx;
            let current = *table_type == active;
            let label = if current {
                format!("{}  {}", table_type.as_str(), tr(app.lang, Msg::Current))
            } else {
                table_type.as_str().to_string()
            };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if current {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(label).style(style)
        })
        .collect::<Vec<_>>();

    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let list = List::new(items)
        .block(
            Block::default()
                .title(tr(app.lang, Msg::TableType))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border)),
        )
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(list, area);
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
