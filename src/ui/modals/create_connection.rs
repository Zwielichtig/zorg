use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;
use crate::ui::utils::{center_rect_at_least, default_block_builder};

// ── internal helpers ────────────────────────────────────────────────────────

fn char_byte_offset(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

fn insert_at_char_pos(s: &mut String, char_idx: usize, c: char) {
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

// ── struct ───────────────────────────────────────────────────────────────────

pub struct CreateConnectionModal {
    pub is_open: bool,
    pub name: String,
    pub username: String,
    pub hostname: String,
    pub port: String,
    pub identity_file: String,
    pub note: String,
    pub active_field: usize,
    pub field_cursor: usize,
    pub editing_connection_id: Option<i32>,
}

impl Default for CreateConnectionModal {
    fn default() -> Self {
        Self {
            is_open: false,
            name: String::new(),
            username: std::env::var("USER").unwrap_or_default(),
            hostname: String::new(),
            port: String::new(),
            identity_file: String::new(),
            note: String::new(),
            active_field: 0,
            field_cursor: 0,
            editing_connection_id: None,
        }
    }
}

impl CreateConnectionModal {
    pub fn load_connection(&mut self, conn: &crate::db::connection::Connection) {
        self.editing_connection_id = conn.id;
        self.name = conn.name.clone();
        self.username = conn.username.clone();
        self.hostname = conn.hostname.clone();
        self.port = conn.port.map(|p| p.to_string()).unwrap_or_default();
        self.identity_file = conn.identity_file.clone().unwrap_or_default();
        self.note = conn.note.clone().unwrap_or_default();
        self.active_field = 0;
        self.field_cursor = self.name.chars().count(); // start at end of name field
        self.is_open = true;
    }

    pub fn reset(&mut self) {
        self.name.clear();
        self.username = std::env::var("USER").unwrap_or_default();
        self.hostname.clear();
        self.port.clear();
        self.identity_file.clear();
        self.note.clear();
        self.active_field = 0;
        self.field_cursor = 0;
        self.editing_connection_id = None;
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.username.is_empty() && !self.hostname.is_empty()
    }

    // ── cursor helpers ────────────────────────────────────────────────────

    pub fn active_field_len(&self) -> usize {
        match self.active_field {
            0 => self.name.chars().count(),
            1 => self.username.chars().count(),
            2 => self.hostname.chars().count(),
            3 => self.port.chars().count(),
            4 => self.identity_file.chars().count(),
            5 => self.note.chars().count(),
            _ => 0,
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.active_field < 6 {
            self.field_cursor = self.field_cursor.saturating_sub(1);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.active_field < 6 {
            self.field_cursor = (self.field_cursor + 1).min(self.active_field_len());
        }
    }

    pub fn cursor_to_end(&mut self) {
        self.field_cursor = self.active_field_len();
    }

    pub fn insert_char_at_cursor(&mut self, c: char) {
        let pos = self.field_cursor;
        match self.active_field {
            0 => insert_at_char_pos(&mut self.name, pos, c),
            1 => insert_at_char_pos(&mut self.username, pos, c),
            2 => insert_at_char_pos(&mut self.hostname, pos, c),
            3 => insert_at_char_pos(&mut self.port, pos, c),
            4 => insert_at_char_pos(&mut self.identity_file, pos, c),
            5 => insert_at_char_pos(&mut self.note, pos, c),
            _ => {}
        }
        if self.active_field < 6 {
            self.field_cursor += 1;
        }
    }

    pub fn backspace_at_cursor(&mut self) {
        if self.field_cursor == 0 {
            return;
        }
        let pos = self.field_cursor;
        match self.active_field {
            0 => remove_char_before(&mut self.name, pos),
            1 => remove_char_before(&mut self.username, pos),
            2 => remove_char_before(&mut self.hostname, pos),
            3 => remove_char_before(&mut self.port, pos),
            4 => remove_char_before(&mut self.identity_file, pos),
            5 => remove_char_before(&mut self.note, pos),
            _ => {}
        }
        self.field_cursor -= 1;
    }

    // ── render ────────────────────────────────────────────────────────────

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        let base_area = center_rect_at_least(64, 64, 56, 24, area);
        f.render_widget(Clear, base_area);

        let popup_area = base_area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 1 });

        let title = if self.editing_connection_id.is_some() {
            "Edit Connection"
        } else {
            "Create Connection"
        };
        let block = default_block_builder(title, false);
        f.render_widget(block.clone(), popup_area);

        let inner_area = block.inner(popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // name
                Constraint::Length(3), // username / hostname
                Constraint::Length(3), // port
                Constraint::Length(3), // identity file
                Constraint::Length(3), // note
                Constraint::Min(1),    // button
            ])
            .split(inner_area);

        let user_host_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .split(chunks[1]);

        let rects = [
            chunks[0],
            user_host_chunks[0],
            user_host_chunks[2],
            chunks[2],
            chunks[3],
            chunks[4],
        ];

        let fields: [(&str, &str, Rect); 6] = [
            ("Name",          self.name.as_str(),          rects[0]),
            ("Username",      self.username.as_str(),      rects[1]),
            ("Hostname",      self.hostname.as_str(),      rects[2]),
            ("Port",          self.port.as_str(),          rects[3]),
            ("Identity File", self.identity_file.as_str(), rects[4]),
            ("Note",          self.note.as_str(),          rects[5]),
        ];

        for (i, (label, value, rect)) in fields.iter().enumerate() {
            let style = if self.active_field == i {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            let field_block = Block::default()
                .borders(Borders::ALL)
                .title(*label)
                .border_style(style);

            // Compute inner area before consuming the block.
            let inner = field_block.inner(*rect);

            if self.active_field == i {
                let byte_off = char_byte_offset(value, self.field_cursor);
                let cursor_col = UnicodeWidthStr::width(&value[..byte_off]) as u16;

                // Scroll so the cursor stays in the visible area.
                let scroll_x = cursor_col.saturating_sub(inner.width.saturating_sub(1));

                let paragraph = Paragraph::new(*value)
                    .block(field_block)
                    .scroll((0, scroll_x));
                f.render_widget(paragraph, *rect);

                let cursor_x = inner.x + cursor_col - scroll_x;
                f.set_cursor_position((cursor_x, inner.y));
            } else {
                let paragraph = Paragraph::new(*value).block(field_block);
                f.render_widget(paragraph, *rect);
            }
        }

        let at_paragraph = Paragraph::new("\n@")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default());
        f.render_widget(at_paragraph, user_host_chunks[1]);

        let button_style = if self.active_field == 6 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let button = Paragraph::new("[ Submit ]")
            .alignment(ratatui::layout::Alignment::Center)
            .style(button_style);

        f.render_widget(button, chunks[5]);
    }
}
