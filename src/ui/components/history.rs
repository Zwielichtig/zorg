use chrono::{Local, TimeZone};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::{App, AppFocus};
use crate::db::history::History;
use crate::ui::utils::default_block_builder;

fn exit_code_label(code: &str) -> &'static str {
    match code {
        "0"       => "Success",
        "1"       => "General error",
        "2"       => "Invalid usage",
        "126"     => "Permission denied",
        "127"     => "Command not found",
        "130"     => "Interrupted",
        "255"     => "Connection failed",
        "unknown" => "Signal killed",
        _         => "Unknown error",
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn time_str(ts: i32) -> String {
    Local
        .timestamp_opt(ts as i64, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "Unknown time".to_string())
}

fn exit_span(code: &str) -> (Span<'static>, String) {
    let text = format!("[{}] {}", code, exit_code_label(code));
    let style = if code == "0" {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };
    (Span::styled(text.clone(), style), text)
}

/// Compute the widest exit string across a slice of history entries so all
/// exit codes can be aligned to the same starting column.
fn max_exit_width(entries: &[History]) -> usize {
    entries
        .iter()
        .map(|h| {
            let s = format!("[{}] {}", h.exit_code, exit_code_label(&h.exit_code));
            UnicodeWidthStr::width(s.as_str())
        })
        .max()
        .unwrap_or(0)
}

/// Truncate a string to at most `max_cols` display columns, appending '…' if cut.
fn truncate_to_cols(s: &str, max_cols: usize) -> (String, usize) {
    if max_cols == 0 {
        return (String::new(), 0);
    }
    let mut out = String::new();
    let mut used = 0usize;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
        if used + cw + 1 > max_cols {
            // +1 reserved for the ellipsis
            out.push('…');
            used += 1;
            return (out, used);
        }
        out.push(ch);
        used += cw;
    }
    (out, used)
}

/// Build a list item for the global history view:
///   name | gap1 | time (fixed col at 1/3) | gap2 | exit (fixed start col)
fn global_item(
    h: &History,
    conn_name: &str,
    inner_width: usize,
    exit_col: usize,
) -> ListItem<'static> {
    let t = time_str(h.started_at);
    let (e_span, _) = exit_span(&h.exit_code);

    // Time is pinned at 1/3 of the panel width.
    let time_col = inner_width / 3;

    // Truncate the name so it never overruns the time column.
    let max_name_cols = time_col.saturating_sub(1);
    let name_w = UnicodeWidthStr::width(conn_name);
    let (display_name, display_name_w) = if name_w > max_name_cols {
        truncate_to_cols(conn_name, max_name_cols)
    } else {
        (conn_name.to_string(), name_w)
    };

    let name_span = Span::styled(
        display_name,
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    );
    let time_span = Span::styled(t.clone(), Style::default().fg(Color::Indexed(244)));
    let time_w = UnicodeWidthStr::width(t.as_str());

    let mut spans: Vec<Span> = vec![name_span];

    // gap1: name → fixed time column
    if time_col > display_name_w {
        spans.push(Span::raw(" ".repeat(time_col - display_name_w)));
    } else {
        spans.push(Span::raw(" "));
    }
    spans.push(time_span);

    // gap2: end of time → fixed exit start column
    let time_end = time_col + time_w;
    if exit_col > time_end {
        spans.push(Span::raw(" ".repeat(exit_col - time_end)));
    } else {
        spans.push(Span::raw(" "));
    }
    spans.push(e_span);

    ListItem::new(Line::from(spans))
}

/// Build a list item for the per-connection history view:
///   time (left) | gap | exit (fixed start col, no name column)
fn connection_item(h: &History, exit_col: usize) -> ListItem<'static> {
    let t = time_str(h.started_at);
    let (e_span, _) = exit_span(&h.exit_code);

    let time_span = Span::styled(t.clone(), Style::default().fg(Color::Indexed(244)));
    let time_w = UnicodeWidthStr::width(t.as_str());

    let mut spans: Vec<Span> = vec![time_span];

    // gap: end of time → fixed exit start column
    if exit_col > time_w {
        spans.push(Span::raw(" ".repeat(exit_col - time_w)));
    } else {
        spans.push(Span::raw(" "));
    }
    spans.push(e_span);

    ListItem::new(Line::from(spans))
}

// ── Public render entry point ─────────────────────────────────────────────────

pub fn render_history(f: &mut Frame, app: &App, area: Rect, dimmed: bool) {
    let inner_width = area.width.saturating_sub(2) as usize;

    match app.focus {
        AppFocus::List => render_connection_history(f, app, area, inner_width, dimmed),
        AppFocus::Search => render_global_history(f, app, area, inner_width, dimmed),
    }
}

/// Per-connection view: shown when the connection list has focus.
fn render_connection_history(
    f: &mut Frame,
    app: &App,
    area: Rect,
    inner_width: usize,
    dimmed: bool,
) {
    let title = if !app.filtered_connections.is_empty()
        && app.selected_connection_index < app.filtered_connections.len()
    {
        let idx = app.filtered_connections[app.selected_connection_index].conn_index;
        format!("History – {}", app.connections[idx].name)
    } else {
        "History".to_string()
    };

    let block = default_block_builder(&title, dimmed);

    if app.connection_history.is_empty() {
        let p = Paragraph::new("No history for this connection").block(block);
        f.render_widget(p, area);
        return;
    }

    // Fixed exit start column based on the widest exit string across all entries.
    let exit_col = inner_width.saturating_sub(max_exit_width(&app.connection_history));

    let items: Vec<ListItem> = app
        .connection_history
        .iter()
        .skip(app.connection_history_scroll)
        .map(|h| connection_item(h, exit_col))
        .collect();

    f.render_widget(List::new(items).block(block), area);
}

/// Global view: shown when the search bar has focus.
fn render_global_history(f: &mut Frame, app: &App, area: Rect, inner_width: usize, dimmed: bool) {
    let block = default_block_builder("History", dimmed);

    if app.recent_history.is_empty() {
        let p = Paragraph::new("No history yet").block(block);
        f.render_widget(p, area);
        return;
    }

    // Fixed exit start column based on the widest exit string across all entries.
    let exit_col = inner_width.saturating_sub(max_exit_width(&app.recent_history));

    let items: Vec<ListItem> = app
        .recent_history
        .iter()
        .skip(app.history_scroll)
        .map(|h| {
            let conn_name = app
                .connections
                .iter()
                .find(|c| c.id == Some(h.connection_id))
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            global_item(h, conn_name, inner_width, exit_col)
        })
        .collect();

    f.render_widget(List::new(items).block(block), area);
}
