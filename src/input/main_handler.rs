use crate::app::{App, AppFocus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// ── char-level string helpers ────────────────────────────────────────────────

fn char_byte_offset(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

fn insert_at_char(s: &mut String, char_idx: usize, c: char) {
    let byte_idx = char_byte_offset(s, char_idx);
    s.insert(byte_idx, c);
}

fn remove_char_before(s: &mut String, char_idx: usize) {
    if char_idx == 0 {
        return;
    }
    let end = char_byte_offset(s, char_idx);
    let start = char_byte_offset(s, char_idx - 1);
    s.drain(start..end);
}

// ── handler ──────────────────────────────────────────────────────────────────

pub fn handle_main_input(app: &mut App, key: KeyEvent) -> bool {
    if key.code == KeyCode::Tab {
        app.focus = if app.focus == AppFocus::Search {
            AppFocus::List
        } else {
            AppFocus::Search
        };
        if app.focus == AppFocus::List {
            app.refresh_connection_history();
        }
        return false;
    }

    match key.code {
        KeyCode::Enter => {
            if app.focus == AppFocus::Search {
                app.focus = AppFocus::List;
                app.refresh_connection_history();
            } else {
                if app.selected_connection_index < app.filtered_connections.len() {
                    let conn_idx = app.filtered_connections[app.selected_connection_index].conn_index;
                    app.pending_ssh_connection = Some(app.connections[conn_idx].clone());
                }
            }
        }
        KeyCode::Left => {
            if app.focus == AppFocus::Search {
                app.input_cursor = app.input_cursor.saturating_sub(1);
            }
        }
        KeyCode::Right => {
            if app.focus == AppFocus::Search {
                let len = app.input.chars().count();
                app.input_cursor = (app.input_cursor + 1).min(len);
            }
        }
        KeyCode::Up => {
            if app.focus == AppFocus::List {
                if app.selected_connection_index > 0 {
                    app.selected_connection_index -= 1;
                    app.refresh_connection_history();
                }
            }
        }
        KeyCode::Down => {
            if app.focus == AppFocus::List {
                if app.selected_connection_index + 1 < app.filtered_connections.len() {
                    app.selected_connection_index += 1;
                    app.refresh_connection_history();
                }
            }
        }
        KeyCode::PageUp => {
            if app.focus == AppFocus::Search {
                app.scroll_history_up(5);
            } else if app.focus == AppFocus::List {
                app.scroll_connection_history_up(5);
            }
        }
        KeyCode::PageDown => {
            if app.focus == AppFocus::Search {
                app.scroll_history_down(5);
            } else if app.focus == AppFocus::List {
                app.scroll_connection_history_down(5);
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
                    // Switch focus to Search and type the key
                    app.focus = AppFocus::Search;
                    app.input_cursor = app.input.chars().count();
                    insert_at_char(&mut app.input, app.input_cursor, key_pressed);
                    app.input_cursor += 1;
                    app.update_search_filter();
                }
            } else {
                insert_at_char(&mut app.input, app.input_cursor, key_pressed);
                app.input_cursor += 1;
                app.update_search_filter();
            }
        }
        KeyCode::Backspace => {
            if app.focus == AppFocus::Search {
                if app.input_cursor > 0 {
                    remove_char_before(&mut app.input, app.input_cursor);
                    app.input_cursor -= 1;
                }
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
