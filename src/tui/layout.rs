use ratatui::layout::{Constraint, Direction, Layout, Rect};

const MAX_CONTENT_WIDTH: u16 = 132;

pub(crate) struct MainLayout {
    pub(crate) disks: Rect,
    pub(crate) partitions: Rect,
    pub(crate) status: Rect,
    pub(crate) menu: Rect,
}

pub(crate) fn split(area: Rect) -> MainLayout {
    let area = centered_content(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Percentage(42),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    MainLayout {
        disks: chunks[0],
        partitions: chunks[1],
        status: chunks[2],
        menu: chunks[3],
    }
}

fn centered_content(area: Rect) -> Rect {
    if area.width <= MAX_CONTENT_WIDTH {
        return area;
    }

    let horizontal_margin = (area.width - MAX_CONTENT_WIDTH) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horizontal_margin),
            Constraint::Length(MAX_CONTENT_WIDTH),
            Constraint::Min(0),
        ])
        .split(area)[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_content_keeps_narrow_area_full_width() {
        let area = Rect::new(0, 0, 100, 20);

        assert_eq!(centered_content(area), area);
    }

    #[test]
    fn centered_content_limits_and_centers_wide_area() {
        let area = Rect::new(0, 0, 200, 20);
        let centered = centered_content(area);

        assert_eq!(centered.width, MAX_CONTENT_WIDTH);
        assert_eq!(centered.x, 34);
    }
}
