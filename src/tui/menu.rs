use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::state::{App, AppLayer, InputMode},
    util::{
        i18n::{tr, Msg},
        size::format_size,
    },
};

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.input_mode {
        InputMode::Normal if app.layer == AppLayer::Main => tr(app.lang, Msg::MainTitle),
        InputMode::Normal => tr(app.lang, Msg::EditTitle),
        InputMode::Size => tr(app.lang, Msg::SizeTitle),
        InputMode::TypePicker => tr(app.lang, Msg::TypePickerTitle),
        InputMode::SetPlan => tr(app.lang, Msg::SetPlanTitle),
        InputMode::WriteConfirm => tr(app.lang, Msg::WriteConfirmTitle),
    };

    let menu_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(menu_block, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    if app.input_mode != InputMode::Normal {
        let prompt = match app.input_mode {
            InputMode::Size => {
                if let Some(adjustment) = app.pending_size_adjustment {
                    Line::from(vec![
                        Span::raw(tr(app.lang, Msg::SizeLabel)),
                        Span::styled(&app.input, Style::default().fg(Color::Yellow)),
                        Span::raw("  "),
                        Span::styled(
                            format!("~ {}", format_size(adjustment.requested_bytes)),
                            Style::default().fg(Color::Cyan),
                        ),
                    ])
                } else if app.input_is_default {
                    Line::from(vec![
                        Span::raw(tr(app.lang, Msg::DefaultLabel)),
                        Span::styled(&app.input, Style::default().fg(Color::DarkGray)),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw(tr(app.lang, Msg::SizeLabel)),
                        Span::styled(&app.input, Style::default().fg(Color::Yellow)),
                    ])
                }
            }
            InputMode::TypePicker => {
                Line::from(vec![Span::raw(tr(app.lang, Msg::TypePickerPrompt))])
            }
            InputMode::SetPlan => Line::from(vec![Span::raw(tr(app.lang, Msg::SetPlanPrompt))]),
            InputMode::WriteConfirm => Line::from(vec![
                Span::raw(tr(app.lang, Msg::ConfirmLabel)),
                Span::styled(&app.input, Style::default().fg(Color::Yellow)),
            ]),
            InputMode::Normal => Line::from(String::new()),
        };
        let paragraph = Paragraph::new(prompt).alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
        return;
    }

    let menus: Vec<&str> = match app.layer {
        AppLayer::Main => vec![
            tr(app.lang, Msg::MainSelect),
            tr(app.lang, Msg::MainRefresh),
            tr(app.lang, Msg::MainSetPlan),
            tr(app.lang, Msg::MainWrite),
            tr(app.lang, Msg::MainQuit),
        ],
        AppLayer::Edit => vec![
            tr(app.lang, Msg::EditNew),
            tr(app.lang, Msg::EditDelete),
            tr(app.lang, Msg::EditType),
            tr(app.lang, Msg::EditCommit),
            tr(app.lang, Msg::EditCancel),
        ],
    };
    let constraints = vec![Constraint::Ratio(1, menus.len() as u32); menus.len()];
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (index, menu) in menus.iter().enumerate() {
        let mut style = if index == app.selected_menu {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        if app.layer == AppLayer::Edit && index != app.selected_menu {
            if *menu == tr(app.lang, Msg::EditCommit) {
                style = Style::default().fg(Color::Yellow);
            } else if *menu == tr(app.lang, Msg::EditCancel) {
                style = Style::default().fg(Color::Red);
            }
        }

        let paragraph = Paragraph::new(format!("[ {menu} ]"))
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[index]);
    }
}
