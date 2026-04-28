use ratatui::{
    layout::Rect,
    widgets::Paragraph,
    Frame,
};
use crate::ui::utils::default_block_builder;

pub fn render_connection_details(f: &mut Frame, area: Rect, dimmed: bool) {
    let block = default_block_builder("Details", dimmed);
    let p = Paragraph::new("Details coming soon...").block(block);
    f.render_widget(p, area);
}
