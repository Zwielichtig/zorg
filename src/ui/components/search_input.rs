use ratatui::{
    layout::Rect,
    style::Style,
    widgets::Paragraph,
    Frame,
};
use unicode_width::UnicodeWidthStr;
use crate::app::App;
use crate::ui::utils::default_block_builder;

pub fn render_search_input(f: &mut Frame, app: &App, area: Rect, dimmed: bool, style: Style) {
    let is_focused = !dimmed && app.focus == crate::app::AppFocus::Search;

    let focus_style = if is_focused {
        Style::default().fg(ratatui::style::Color::Yellow)
    } else {
        style
    };

    let block = default_block_builder("Search Input", dimmed).border_style(focus_style);
    let inner = block.inner(area);

    if is_focused {
        // Byte offset of the cursor char within the string.
        let byte_off: usize = app.input
            .char_indices()
            .nth(app.input_cursor)
            .map(|(b, _)| b)
            .unwrap_or(app.input.len());

        // Column width of the text to the left of the cursor.
        let cursor_col = UnicodeWidthStr::width(&app.input[..byte_off]) as u16;

        // Scroll so the cursor stays within the visible area.
        // scroll_x = how many columns of the string are hidden on the left.
        let scroll_x = cursor_col.saturating_sub(inner.width.saturating_sub(1));

        let p = Paragraph::new(app.input.as_str())
            .block(block)
            .scroll((0, scroll_x));
        f.render_widget(p, area);

        let cursor_x = inner.x + cursor_col - scroll_x;
        f.set_cursor_position((cursor_x, inner.y));
    } else {
        let p = Paragraph::new(app.input.as_str()).block(block);
        f.render_widget(p, area);
    }
}
