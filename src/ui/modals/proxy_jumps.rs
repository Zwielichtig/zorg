use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Clear, Paragraph, List, ListItem, ListState},
    Frame,
};
use std::cell::RefCell;
use crate::ui::utils::{center_rect_at_least, default_block_builder};
use crate::db::connection::Connection;
use crate::db::hop::ConnectionHop;
use diesel::SqliteConnection;

pub struct ProxyJumpsModal {
    pub is_open: bool,
    pub source_connection_id: Option<i32>,
    pub source_connection_name: String,
    pub available_connections: Vec<Connection>,
    pub selected_jump_ids: Vec<i32>,
    pub list_state: RefCell<ListState>,
    pub message: Option<String>,
}

impl Default for ProxyJumpsModal {
    fn default() -> Self {
        Self {
            is_open: false,
            source_connection_id: None,
            source_connection_name: String::new(),
            available_connections: Vec::new(),
            selected_jump_ids: Vec::new(),
            list_state: RefCell::new(ListState::default()),
            message: None,
        }
    }
}

impl ProxyJumpsModal {
    pub fn open(&mut self, db: &mut SqliteConnection, source_conn: &Connection) {
        self.source_connection_id = source_conn.id;
        self.source_connection_name = source_conn.name.clone();
        
        // Fetch all possible jumps (exclude self)
        if let Ok(all_conns) = Connection::get_all(db) {
            self.available_connections = all_conns.into_iter()
                .filter(|c| c.id != source_conn.id)
                .collect();
        }
        
        // Fetch existing jumps
        self.selected_jump_ids.clear();
        if let Some(sid) = source_conn.id {
             if let Ok(jumps) = ConnectionHop::get_jumps(db, sid) {
                 self.selected_jump_ids = jumps.into_iter().filter_map(|c| c.id).collect();
             }
        }
        
        self.message = None;
        if !self.available_connections.is_empty() {
            self.list_state.borrow_mut().select(Some(0));
        } else {
            self.list_state.borrow_mut().select(None);
        }
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.source_connection_id = None;
        self.source_connection_name.clear();
        self.message = None;
    }

    pub fn toggle_selected(&mut self) {
        if let Some(idx) = self.list_state.borrow().selected() {
            if let Some(target_id) = self.available_connections[idx].id {
                if let Some(pos) = self.selected_jump_ids.iter().position(|&id| id == target_id) {
                    self.selected_jump_ids.remove(pos);
                } else {
                    self.selected_jump_ids.push(target_id);
                }
            }
        }
    }
    
    // allow ordering
    pub fn move_up(&mut self) {
        if let Some(idx) = self.list_state.borrow().selected() {
            if let Some(target_id) = self.available_connections[idx].id {
                if let Some(pos) = self.selected_jump_ids.iter().position(|&id| id == target_id) {
                    if pos > 0 {
                        self.selected_jump_ids.swap(pos, pos - 1);
                    }
                }
            }
        }
    }
    
    pub fn move_down(&mut self) {
        if let Some(idx) = self.list_state.borrow().selected() {
            if let Some(target_id) = self.available_connections[idx].id {
                if let Some(pos) = self.selected_jump_ids.iter().position(|&id| id == target_id) {
                    if pos < self.selected_jump_ids.len() - 1 {
                        self.selected_jump_ids.swap(pos, pos + 1);
                    }
                }
            }
        }
    }

    pub fn save(&mut self, db: &mut SqliteConnection) {
        if let Some(sid) = self.source_connection_id {
            match ConnectionHop::set_jumps(db, sid, self.selected_jump_ids.clone()) {
                Ok(_) => self.message = Some("Saved successfully!".to_string()),
                Err(_) => self.message = Some("Failed to save.".to_string()),
            }
        }
    }

    pub fn next(&mut self) {
        if self.available_connections.is_empty() { return; }
        let i = match self.list_state.borrow().selected() {
            Some(i) => if i >= self.available_connections.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.borrow_mut().select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.available_connections.is_empty() { return; }
        let i = match self.list_state.borrow().selected() {
            Some(i) => if i == 0 { self.available_connections.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.borrow_mut().select(Some(i));
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.is_open { return; }

        let base_area = center_rect_at_least(64, 64, 54, 15, area);
        f.render_widget(Clear, base_area);

        let popup_area = base_area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 1 });
        let title = format!("Proxy Jumps for: {}", self.source_connection_name);
        let block = default_block_builder(&title, false);
        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // list
                Constraint::Length(1), // message
                Constraint::Length(1), // help
            ])
            .split(inner_area);

        let items: Vec<ListItem> = self.available_connections.iter().map(|c| {
            let mut style = Style::default();
            let mut prefix = "[ ]";
            let mut suffix = String::new();
            
            if let Some(cid) = c.id {
                if let Some(pos) = self.selected_jump_ids.iter().position(|&id| id == cid) {
                    prefix = "[x]";
                    suffix = format!("(Hop {})", pos + 1);
                    style = style.fg(Color::LightGreen);
                } else {
                    style = style.fg(Color::White);
                }
            }

            let text = format!("{} {} {}", prefix, c.name, suffix);
            ListItem::new(text).style(style)
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Available Connections"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[0], &mut *self.list_state.borrow_mut());

        let msg = self.message.as_deref().unwrap_or("");
        f.render_widget(Paragraph::new(msg).style(Style::default().fg(Color::Yellow)), chunks[1]);
        
        let help = "Space: Toggle | +/-: Move Order | Enter: Save | Esc: Close";
        f.render_widget(Paragraph::new(help).style(Style::default().fg(Color::Gray)), chunks[2]);
    }
}
