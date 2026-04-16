pub mod main_handler;
pub mod modal_handler;

use crate::app::App;
use crossterm::event::{KeyEvent, MouseEvent};
use main_handler::handle_main_input;
use modal_handler::handle_modal_input;

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    if app.create_connection_modal.is_open {
        handle_modal_input(app, key)
    } else if app.keys_modal.is_open {
        handle_modal_input(app, key)
    } else if app.delete_connection_modal.is_open {
        handle_modal_input(app, key)
    } else if app.proxy_jumps_modal.is_open {
        handle_modal_input(app, key)
    } else {
        handle_main_input(app, key)
    }
}

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> bool {
    if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
        // TODO: optimize mouse support, currently very basic heuristic: if clicking in top top few rows, focus search, otherwise list
        if mouse.row < 4 {
            app.focus = crate::app::AppFocus::Search;
        } else {
            app.focus = crate::app::AppFocus::List;
        }
    }
    false
}
