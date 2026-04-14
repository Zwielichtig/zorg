pub mod layout;
pub mod modals;
pub mod utils;
pub mod components;

use crate::app::App;
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    layout::draw(f, app);
    app.create_connection_modal.render(f, f.area());
    app.keys_modal.render(f, f.area());
}
