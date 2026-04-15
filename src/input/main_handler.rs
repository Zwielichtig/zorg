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
                if app.selected_connection_index < app.filtered_connections.len() {
                    let conn_idx = app.filtered_connections[app.selected_connection_index].conn_index;
                    app.pending_ssh_connection = Some(app.connections[conn_idx].clone());
                }
            }
        }
        KeyCode::Up => {
            if app.focus == AppFocus::List {
                if app.selected_connection_index > 0 {
                    app.selected_connection_index -= 1;
                }
            }
        }
        KeyCode::Down => {
            if app.focus == AppFocus::List {
                if app.selected_connection_index + 1 < app.filtered_connections.len() {
                    app.selected_connection_index += 1;
                }
            }
        }
        KeyCode::Char(key_pressed) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'c' {
                return true; // request exit
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'n' {
                app.create_connection_modal.is_open = true;
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'd' {
                if app.focus == AppFocus::List && app.selected_connection_index < app.filtered_connections.len() {
                    let conn_idx = app.filtered_connections[app.selected_connection_index].conn_index;
                    let conn = &app.connections[conn_idx];
                    if let Some(id) = conn.id {
                        app.delete_connection_modal.open(id, conn.name.clone());
                    }
                }
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'e' {
                if app.focus == AppFocus::List && app.selected_connection_index < app.filtered_connections.len() {
                    let conn_idx = app.filtered_connections[app.selected_connection_index].conn_index;
                    let conn = app.connections[conn_idx].clone();
                    app.create_connection_modal.load_connection(&conn);
                }

            // TODO: Add functionalty for password protected keys + rework ui
            // } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'k' {
            //     app.keys_modal.open();
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'p' {
                if app.focus == AppFocus::List && app.selected_connection_index < app.filtered_connections.len() {
                    let conn_idx = app.filtered_connections[app.selected_connection_index].conn_index;
                    let conn = app.connections[conn_idx].clone();
                    app.proxy_jumps_modal.open(&mut app.db, &conn);
                }
            } else if key.modifiers.contains(KeyModifiers::CONTROL) && key_pressed == 'h' {
                app.show_help_modal = !app.show_help_modal;
            } else if app.focus == AppFocus::List {
                if key_pressed == 'f' {
                    if app.selected_connection_index < app.filtered_connections.len() {
                        let conn_idx = app.filtered_connections[app.selected_connection_index].conn_index;
                        if let Some(id) = app.connections[conn_idx].id {
                            let mut db_conn = app.db_conn();
                            if let Ok(updated) = crate::db::connection::Connection::toggle_favorite(&mut db_conn, id) {
                                app.connections[conn_idx] = updated;
                                app.update_search_filter();
                            }
                        }
                    }
                } else {
                    app.focus = AppFocus::Search;
                    app.input.push(key_pressed);
                    app.update_search_filter();
                }
            } else {
                app.input.push(key_pressed);
                app.update_search_filter();
            }
        }
        KeyCode::Backspace => {
            if app.focus == AppFocus::Search {
                app.input.pop();
                app.update_search_filter();
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
