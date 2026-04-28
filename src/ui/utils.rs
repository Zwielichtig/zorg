use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, BorderType},
};

pub fn center_rect_at_least(
    percent_x: u16,
    percent_y: u16,
    min_w: u16,
    min_h: u16,
    r: Rect,
) -> Rect {
    let w = ((r.width as u32 * percent_x as u32 / 100) as u16)
        .max(min_w)
        .min(r.width);
    let h = ((r.height as u32 * percent_y as u32 / 100) as u16)
        .max(min_h)
        .min(r.height);
    let x = r.x + r.width.saturating_sub(w) / 2;
    let y = r.y + r.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}

pub fn default_block_builder(title: &str, dimmed: bool) -> Block<'static> {
    let color = if dimmed {
        ratatui::style::Color::Indexed(244)
    } else {
        ratatui::style::Color::Reset
    };
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title.to_string())
        .border_style(ratatui::style::Style::default().fg(color))
        .title_style(ratatui::style::Style::default().fg(color))
}
