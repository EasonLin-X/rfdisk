use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    app::state::App,
    util::i18n::{tr, Msg},
};

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(area, 62, 36);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Set(plan) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(block, popup);

    let inner = popup.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    let text = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            tr(app.lang, Msg::SetPlanPrompt),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(tr(app.lang, Msg::SetPlanAlphaFocus1)),
        Line::from(tr(app.lang, Msg::SetPlanAlphaFocus2)),
        Line::from(""),
        Line::from(tr(app.lang, Msg::SetPlanReturnLater)),
        Line::from(""),
        Line::from(vec![
            Span::raw(tr(app.lang, Msg::Press)),
            Span::styled("Q", Style::default().fg(Color::Cyan)),
            Span::raw(tr(app.lang, Msg::Or)),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(tr(app.lang, Msg::ToReturn)),
        ]),
    ]);
    frame.render_widget(text, inner);
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
