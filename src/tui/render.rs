use ratatui::Frame;

use crate::{
    app::state::{App, InputMode},
    tui,
};

pub(crate) fn ui(frame: &mut Frame, app: &mut App) {
    let layout = tui::layout::split(frame.area());

    tui::disk_table::render(frame, layout.disks, app);
    tui::partition_table::render(frame, layout.partitions, app);
    tui::status::render(frame, layout.status, app);
    tui::menu::render(frame, layout.menu, app);
    if app.input_mode == InputMode::TypePicker {
        tui::popup::render_type_picker(frame, frame.area(), app);
    } else if app.input_mode == InputMode::SetPlan {
        tui::set_plan::render(frame, frame.area(), app);
    }
}
