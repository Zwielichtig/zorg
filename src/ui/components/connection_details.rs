use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, AppFocus, ProxyInfo};
use crate::ui::utils::default_block_builder;

pub fn render_connection_details(f: &mut Frame, app: &App, area: Rect, dimmed: bool) {
    let block = default_block_builder("Details", dimmed);

    if app.focus != AppFocus::List
        || app.filtered_connections.is_empty()
        || app.selected_connection_index >= app.filtered_connections.len()
    {
        let p = Paragraph::new("Select a connection to view details.").block(block);
        f.render_widget(p, area);
        return;
    }

    let conn_idx = app.filtered_connections[app.selected_connection_index].conn_index;
    let conn = &app.connections[conn_idx];

    let mut lines = Vec::new();

    // Key-value helper
    let mut add_kv = |key: &str, val: &str| {
        lines.push(Line::from(vec![
            Span::styled(format!("{:width$}", key, width = 12), Style::default().fg(Color::DarkGray)),
            Span::styled(val.to_string(), Style::default().fg(Color::White)),
        ]));
    };

    add_kv("Username:", &conn.username);
    add_kv("Hostname:", &conn.hostname);
    if let Some(p) = conn.port {
        add_kv("Port:", &p.to_string());
    } else {
        add_kv("Port:", "22 (default)");
    }
    if let Some(id_file) = &conn.identity_file {
        add_kv("Identity:", id_file);
    }
    if let Some(n) = &conn.note {
        add_kv("Note:", n);
    }
    lines.push(Line::from(""));

    // Draw proxy diagram
    let current_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let jump_style = Style::default().fg(Color::LightBlue);
    let dest_style = Style::default().fg(Color::Green);
    let arrow_style = Style::default().fg(Color::DarkGray);

    match &app.selected_proxy_info {
        ProxyInfo::None => {}
        ProxyInfo::Destination { hops } => {
            lines.push(Line::from(Span::styled("Proxy Chain:", Style::default().fg(Color::Gray))));
            let mut chain = Vec::new();
            for hop in hops {
                chain.push(Span::styled(hop.name.clone(), jump_style));
                chain.push(Span::styled(" -> ", arrow_style));
            }
            chain.push(Span::styled(conn.name.clone(), current_style));
            lines.push(Line::from(chain));
        }
        ProxyInfo::JumpHost { chains } => {
            lines.push(Line::from(Span::styled("Used as Jump Host for:", Style::default().fg(Color::Gray))));
            for (hops, dest) in chains {
                let mut chain = Vec::new();
                chain.push(Span::raw("  "));
                for hop in hops {
                    if hop.id == conn.id {
                        chain.push(Span::styled(hop.name.clone(), current_style));
                    } else {
                        chain.push(Span::styled(hop.name.clone(), jump_style));
                    }
                    chain.push(Span::styled(" -> ", arrow_style));
                }
                chain.push(Span::styled(dest.name.clone(), dest_style));
                lines.push(Line::from(chain));
            }
        }
        ProxyInfo::Both { hops, chains } => {
            lines.push(Line::from(Span::styled("Proxy Chain (Destination):", Style::default().fg(Color::Gray))));
            let mut chain = Vec::new();
            for hop in hops {
                chain.push(Span::styled(hop.name.clone(), jump_style));
                chain.push(Span::styled(" -> ", arrow_style));
            }
            chain.push(Span::styled(conn.name.clone(), current_style));
            lines.push(Line::from(chain));
            
            lines.push(Line::from(""));
            
            lines.push(Line::from(Span::styled("Used as Jump Host for:", Style::default().fg(Color::Gray))));
            for (dest_hops, dest) in chains {
                let mut chain = Vec::new();
                chain.push(Span::raw("  "));
                for hop in dest_hops {
                    if hop.id == conn.id {
                        chain.push(Span::styled(hop.name.clone(), current_style));
                    } else {
                        chain.push(Span::styled(hop.name.clone(), jump_style));
                    }
                    chain.push(Span::styled(" -> ", arrow_style));
                }
                chain.push(Span::styled(dest.name.clone(), dest_style));
                lines.push(Line::from(chain));
            }
        }
    }

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false }); // trim: false so indentation is kept
    f.render_widget(p, area);
}
