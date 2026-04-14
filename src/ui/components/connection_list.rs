use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};
use crate::app::App;
use crate::ui::utils::default_block_builder;

pub fn render_search_results(f: &mut Frame, app: &App, area: Rect, dimmed: bool, style: Style) {
    let focus_style = if app.focus == crate::app::AppFocus::List {
        Style::default().fg(ratatui::style::Color::Yellow)
    } else {
        style
    };
    let block = default_block_builder("Connections", dimmed).border_style(focus_style);

    let items: Vec<ListItem> = app
        .filtered_connections
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let c = &app.connections[result.conn_index];
            let fav_icon = if c.is_favorite { "★ " } else { "  " };
            
            let base_style = if i == app.selected_connection_index && app.focus == crate::app::AppFocus::List {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            
            // use cyan to highlight matched characters
            let highlight_style = base_style.patch(Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD));
            
            let mut spans = vec![Span::styled(fav_icon, base_style)];
            
            let push_marked_string = |spans: &mut Vec<Span>, s: &str, indices: &[usize]| {
                for (idx, ch) in s.chars().enumerate() {
                    let style = if indices.contains(&idx) {
                        highlight_style
                    } else {
                        base_style
                    };
                    spans.push(Span::styled(ch.to_string(), style));
                }
            };
            
            push_marked_string(&mut spans, &c.name, &result.name_indices);
            spans.push(Span::styled(" (", base_style));
            push_marked_string(&mut spans, &c.username, &result.username_indices);
            spans.push(Span::styled("@", base_style));
            push_marked_string(&mut spans, &c.hostname, &result.hostname_indices);
            spans.push(Span::styled(")", base_style));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
