use crate::app::{App, AppFocus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_main_input(app: &mut App, key: KeyEvent) -> bool {
    if key.code == KeyCode::Tab {
        app.focus = if app.focus == AppFocus::Search {
            AppFocus::List
        } else {
            AppFocus::Search
        };
        return false;
    }

    match key.code {
        KeyCode::Enter => {
            if app.focus == AppFocus::Search {
                app.focus = AppFocus::List;
            } else {
                if app.selected_connection_index < app.connections.len() {
                    app.pending_ssh_connection = Some(app.connections[app.selected_connection_index].clone());
                }
            }
        }
        KeyCode::Up => {
            app.focus = AppFocus::List;
            if app.selected_connection_index > 0 {
                app.selected_connection_index -= 1;
            }
        }
        KeyCode::Down => {
            app.focus = AppFocus::List;
            if app.selected_connection_index + 1 < app.connections.len() {
                app.selected_connection_index += 1;
            }
        }
        KeyCode::Char(key_pressed) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'c' {
                return true; // Request exit
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'n' {
                app.create_connection_modal.is_open = true;
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'e' {
                if app.focus == AppFocus::List && app.selected_connection_index < app.connections.len() {
                    let conn = app.connections[app.selected_connection_index].clone();
                    app.create_connection_modal.load_connection(&conn);
                }
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'k' {
                app.keys_modal.open();
            } else if app.focus == AppFocus::List {
                if key_pressed == 'f' {
                    if app.selected_connection_index < app.connections.len() {
                        if let Some(id) = app.connections[app.selected_connection_index].id {
                            let mut db_conn = app.db_conn();
                            if let Ok(updated) = crate::db::connection::Connection::toggle_favorite(&mut db_conn, id) {
                                app.connections[app.selected_connection_index] = updated;
                            }
                        }
                    }
                } else if key_pressed == '?' {
                    app.show_help_modal = !app.show_help_modal;
                } else {
                    app.focus = AppFocus::Search;
                    app.input.push(key_pressed);
                }
            } else {
                app.input.push(key_pressed);
            }
        }
        KeyCode::Backspace => {
            if app.focus == AppFocus::Search {
                app.input.pop();
            }
        }
        KeyCode::Esc => {
            if app.show_help_modal {
                app.show_help_modal = false;
            } else {
                return true; // Request exit
            }
        }
        _ => {}
    }
    false
}
