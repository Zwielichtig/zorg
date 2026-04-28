use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::ui::utils::default_block_builder;

pub fn render_connection_list(f: &mut Frame, app: &App, area: Rect, dimmed: bool, style: Style) {
    let focus_style = if app.focus == crate::app::AppFocus::List {
        Style::default().fg(Color::Yellow)
    } else {
        style
    };

    let block = default_block_builder("Connections", dimmed).border_style(focus_style);

    // account for borders (left + right)
    let inner_width = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = app
        .filtered_connections
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let c = &app.connections[result.conn_index];

            let fav_icon = if c.is_favorite { "★ " } else { "  " };

            let is_proxy = app.is_proxy(c);
            let has_proxy = app.has_proxy(c);

            let base_style = if i == app.selected_connection_index
                && app.focus == crate::app::AppFocus::List
            {
                Style::default().fg(Color::Yellow)
            } else if has_proxy {
                Style::default().fg(Color::LightBlue)
            } else if is_proxy {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let highlight_style = base_style.patch(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            );

            let mut spans = vec![Span::styled(fav_icon, base_style)];

            let name_idx: std::collections::HashSet<_> =
                result.name_indices.iter().copied().collect();
            let user_idx: std::collections::HashSet<_> =
                result.username_indices.iter().copied().collect();
            let host_idx: std::collections::HashSet<_> =
                result.hostname_indices.iter().copied().collect();

            let push_marked_string =
                |spans: &mut Vec<Span>, s: &str, indices: &std::collections::HashSet<usize>| {
                    for (idx, ch) in s.chars().enumerate() {
                        let style = if indices.contains(&idx) {
                            highlight_style
                        } else {
                            base_style
                        };
                        spans.push(Span::styled(ch.to_string(), style));
                    }
                };

            push_marked_string(&mut spans, &c.name, &name_idx);
            spans.push(Span::styled(" (", base_style));
            push_marked_string(&mut spans, &c.username, &user_idx);
            spans.push(Span::styled("@", base_style));
            push_marked_string(&mut spans, &c.hostname, &host_idx);
            spans.push(Span::styled(")", base_style));

            let mut line = Line::from(spans);

            if has_proxy {
                let indicator = "Proxy ";
                let indicator_width = UnicodeWidthStr::width(indicator);
                let content_width = line.width();

                if content_width + indicator_width <= inner_width {
                    let padding = inner_width - content_width - indicator_width;
                    line.spans.push(Span::raw(" ".repeat(padding)));
                    line.spans.push(Span::styled(indicator, base_style));
                }
            }

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}