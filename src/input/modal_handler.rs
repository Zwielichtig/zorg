use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_modal_input(app: &mut App, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return true; // request exit
    }

    if app.keys_modal.is_open {
        match key.code {
            KeyCode::Esc => app.keys_modal.close(),
            KeyCode::Up => app.keys_modal.previous(),
            KeyCode::Down => app.keys_modal.next(),
            KeyCode::Enter => app.keys_modal.add_selected_to_agent(),
            KeyCode::Char('a') => app.keys_modal.start_ssh_agent(),
            _ => {}
        }
    }

    if app.proxy_jumps_modal.is_open {
        match key.code {
            KeyCode::Esc => app.proxy_jumps_modal.close(),
            KeyCode::Up => app.proxy_jumps_modal.previous(),
            KeyCode::Down => app.proxy_jumps_modal.next(),
            KeyCode::Char(' ') => app.proxy_jumps_modal.toggle_selected(),
            KeyCode::Char('+') => app.proxy_jumps_modal.move_up(),
            KeyCode::Char('-') => app.proxy_jumps_modal.move_down(),
            KeyCode::Enter => {
                app.proxy_jumps_modal.save(&mut app.db);
                app.refresh_connections();
            }
            _ => {}
        }
        return false;
    }

    if app.delete_connection_modal.is_open {
        match key.code {
            KeyCode::Esc => app.delete_connection_modal.close(),
            KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => {
                app.delete_connection_modal.selected_yes = !app.delete_connection_modal.selected_yes;
            }
            KeyCode::Enter => {
                if app.delete_connection_modal.selected_yes {
                    app.delete_connection();
                } else {
                    app.delete_connection_modal.close();
                }
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Esc => {
            app.close_modal();
        }
        KeyCode::Tab => {
            app.create_connection_modal.active_field =
                (app.create_connection_modal.active_field + 1) % 7;
            app.create_connection_modal.cursor_to_end();
        }
        KeyCode::BackTab => {
            if app.create_connection_modal.active_field == 0 {
                app.create_connection_modal.active_field = 6;
            } else {
                app.create_connection_modal.active_field -= 1;
            }
            app.create_connection_modal.cursor_to_end();
        }
        KeyCode::Left => {
            app.create_connection_modal.move_cursor_left();
        }
        KeyCode::Right => {
            app.create_connection_modal.move_cursor_right();
        }
        KeyCode::Char(c) => {
            if app.create_connection_modal.active_field < 6 {
                app.create_connection_modal.insert_char_at_cursor(c);
            }
        }
        KeyCode::Backspace => {
            app.create_connection_modal.backspace_at_cursor();
        }
        KeyCode::Enter => {
            if app.create_connection_modal.active_field == 6
                && app.create_connection_modal.is_valid()
            {
                app.submit_connection();
            }
        }
        _ => {}
    }
    false
}
