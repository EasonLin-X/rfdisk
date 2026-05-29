use crossterm::event::{KeyCode, KeyEventKind};

use crate::{
    app::state::{App, AppLayer, Focus, InputMode},
    util::i18n::Lang,
};

pub(crate) fn handle_key_event(app: &mut App, code: KeyCode, kind: KeyEventKind) -> bool {
    if kind == KeyEventKind::Release {
        return false;
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_key(app, code, kind),
        InputMode::Size => {
            if kind == KeyEventKind::Press {
                handle_size_key(app, code);
            }
            false
        }
        InputMode::TypePicker => {
            if kind == KeyEventKind::Press {
                handle_type_picker_key(app, code);
            }
            false
        }
        InputMode::SetPlan => {
            if kind == KeyEventKind::Press {
                handle_set_plan_key(app, code);
            }
            false
        }
        InputMode::WriteConfirm => {
            if kind == KeyEventKind::Press {
                handle_confirm_key(app, code);
            }
            false
        }
    }
}

fn handle_normal_key(app: &mut App, code: KeyCode, kind: KeyEventKind) -> bool {
    if kind == KeyEventKind::Repeat {
        if matches!(code, KeyCode::Char('w') | KeyCode::Char('W'))
            && app.layer == AppLayer::Edit
            && app.selected_menu == 1
        {
            app.delete_selected_partition();
        }
        return false;
    }

    match code {
        KeyCode::Left => {
            app.selected_menu = app.selected_menu.saturating_sub(1);
            update_menu_hint(app);
        }
        KeyCode::Right => {
            app.selected_menu = (app.selected_menu + 1).min(app.menu_len().saturating_sub(1));
            update_menu_hint(app);
        }
        KeyCode::Tab => handle_tab(app),
        KeyCode::Up => match app.focus {
            Focus::Disks => {
                if app.can_move_disks() {
                    app.current_disk_idx = app.current_disk_idx.saturating_sub(1);
                } else {
                    app.status =
                        "Pending changes exist. Commit or Cancel before selecting another disk."
                            .to_string();
                }
                app.clamp_selection();
            }
            Focus::Partitions => {
                app.current_partition_idx = app.current_partition_idx.saturating_sub(1);
                app.partition_table_state
                    .select(Some(app.current_partition_idx));
            }
        },
        KeyCode::Down => match app.focus {
            Focus::Disks => {
                if app.can_move_disks() {
                    if !app.disks.is_empty() {
                        app.current_disk_idx = (app.current_disk_idx + 1).min(app.disks.len() - 1);
                    }
                } else {
                    app.status =
                        "Pending changes exist. Commit or Cancel before selecting another disk."
                            .to_string();
                }
                app.clamp_selection();
            }
            Focus::Partitions => {
                let count = app.partition_row_count();
                if count > 0 {
                    app.current_partition_idx = (app.current_partition_idx + 1).min(count - 1);
                    app.partition_table_state
                        .select(Some(app.current_partition_idx));
                }
            }
        },
        KeyCode::Enter => return handle_menu_action(app),
        KeyCode::Char('w') | KeyCode::Char('W')
            if app.layer == AppLayer::Edit && app.selected_menu == 1 =>
        {
            app.status =
                "Hold W on Delete to continuously delete partitions from the current cursor."
                    .to_string();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => handle_refresh_shortcut(app),
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            if handle_escape_or_q(app) {
                return true;
            }
        }
        _ => {}
    }

    false
}

fn handle_tab(app: &mut App) {
    match app.layer {
        AppLayer::Main => app.enter_edit_layer(),
        AppLayer::Edit => app.leave_edit_layer(),
    }
}

fn handle_refresh_shortcut(app: &mut App) {
    match app.layer {
        AppLayer::Main => app.refresh(true),
        AppLayer::Edit => {
            app.status = "Commit or Cancel before refreshing disks.".to_string();
        }
    }
}

fn handle_escape_or_q(app: &mut App) -> bool {
    match app.layer {
        AppLayer::Main => true,
        AppLayer::Edit => {
            app.cancel_draft();
            false
        }
    }
}

fn update_menu_hint(app: &mut App) {
    if app.input_mode == InputMode::Normal && app.layer == AppLayer::Edit && app.selected_menu == 1
    {
        app.status =
            "Delete removes the selected partition. Hold W here to continuously delete partitions."
                .to_string();
    }
}

fn handle_size_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => app.add_new_partition_from_input(),
        KeyCode::Esc => {
            app.input.clear();
            app.input_is_default = false;
            app.pending_size_adjustment = None;
            app.pending_new_start_sector = None;
            app.pending_new_available_bytes = 0;
            app.input_mode = InputMode::Normal;
            app.status = "New partition input canceled. Draft is still pending.".to_string();
        }
        KeyCode::Backspace => {
            if app.input_is_default {
                app.input.clear();
                app.input_is_default = false;
                return;
            }
            app.pending_size_adjustment = None;
            app.input.pop();
        }
        KeyCode::Char(ch) if ch.is_ascii_alphanumeric() || ch == '.' => {
            if app.input_is_default || app.pending_size_adjustment.is_some() {
                app.input.clear();
                app.input_is_default = false;
                app.pending_size_adjustment = None;
            }
            app.input.push(ch);
        }
        _ => {}
    }
}

fn handle_confirm_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => app.finish_write_confirmation(),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.input.clear();
            app.input_mode = InputMode::Normal;
            app.layer = AppLayer::Main;
            app.focus = Focus::Disks;
            app.status = "Write confirmation canceled. Drafts are still pending.".to_string();
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(ch) if ch.is_ascii_alphabetic() || ch == ' ' => {
            app.input.push(ch.to_ascii_lowercase());
        }
        _ => {}
    }
}

fn handle_type_picker_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Left | KeyCode::Right => app.toggle_type_picker_column(),
        KeyCode::Up => app.move_type_picker_selection(-1),
        KeyCode::Down => app.move_type_picker_selection(1),
        KeyCode::Enter => app.apply_type_picker_selection(),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => app.close_type_picker(),
        _ => {}
    }
}

fn handle_set_plan_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down | KeyCode::Enter => {
            app.status = match app.lang {
                Lang::En => "Set(plan) is still under development.".to_string(),
                Lang::ZhCn => "设置计划还在研发中。".to_string(),
            };
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => app.close_set_plan(),
        _ => {}
    }
}

fn handle_menu_action(app: &mut App) -> bool {
    match app.layer {
        AppLayer::Main => match app.selected_menu {
            0 => app.enter_edit_layer(),
            1 => app.refresh(true),
            2 => app.enter_set_plan(),
            3 => app.start_write_all(),
            4 => return true,
            _ => {}
        },
        AppLayer::Edit => match app.selected_menu {
            0 => app.start_new_partition(),
            1 => app.delete_selected_partition(),
            2 => app.enter_type_picker(),
            3 => app.commit_active_draft(),
            4 => app.cancel_draft(),
            _ => {}
        },
    }

    false
}
