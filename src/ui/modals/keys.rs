use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Clear, Paragraph, List, ListItem, ListState},
    Frame,
};
use std::cell::RefCell;
use crate::ui::utils::{center_rect_at_least, default_block_builder};
use crate::ssh::keys::{SshKeyInfo, get_available_keys};
use crate::ssh::agent::{is_agent_running, add_key_to_agent, start_agent};

pub struct KeysModal {
    pub is_open: bool,
    pub keys: Vec<SshKeyInfo>,
    pub list_state: RefCell<ListState>,
    pub agent_running: bool,
    pub message: Option<String>,
}

impl Default for KeysModal {
    fn default() -> Self {
        Self {
            is_open: false,
            keys: Vec::new(),
            list_state: RefCell::new(ListState::default()),
            agent_running: false,
            message: None,
        }
    }
}

impl KeysModal {
    pub fn refresh(&mut self) {
        self.keys = get_available_keys();
        self.agent_running = is_agent_running();
        if self.keys.is_empty() {
            self.list_state.borrow_mut().select(None);
        } else if self.list_state.borrow().selected().is_none() {
            self.list_state.borrow_mut().select(Some(0));
        } else {
            // keep bounds
            let max = self.keys.len().saturating_sub(1);
            let current = self.list_state.borrow().selected().unwrap();
            if current > max {
                self.list_state.borrow_mut().select(Some(max));
            }
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.message = None;
        self.refresh();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.message = None;
    }

    pub fn add_selected_to_agent(&mut self) {
        if let Some(i) = self.list_state.borrow().selected() {
            let key = &self.keys[i];
            if !key.is_private {
                self.message = Some("Only private keys can be added to the agent".to_string());
                return;
            }
            if !self.agent_running {
                self.message = Some("SSH Agent is not running".to_string());
                return;
            }
            match add_key_to_agent(&key.path) {
                Ok(true) => self.message = Some(format!("Successfully added {}", key.path.file_name().unwrap_or_default().to_string_lossy())),
                Ok(false) => self.message = Some("Failed to add key to agent".to_string()),
                Err(e) => self.message = Some(format!("Error: {}", e)),
            }
        }
    }

    pub fn start_ssh_agent(&mut self) {
        match start_agent() {
            Ok(true) => {
                self.message = Some("SSH Agent started successfully".to_string());
                self.agent_running = true;
            }
            Ok(false) => {
                self.message = Some("Failed to start SSH Agent".to_string());
            }
            Err(e) => {
                self.message = Some(format!("Error starting agent: {}", e));
            }
        }
    }

    pub fn next(&mut self) {
        if self.keys.is_empty() {
            return;
        }
        let i = match self.list_state.borrow().selected() {
            Some(i) => if i >= self.keys.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.borrow_mut().select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.keys.is_empty() {
            return;
        }
        let i = match self.list_state.borrow().selected() {
            Some(i) => if i == 0 { self.keys.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.borrow_mut().select(Some(i));
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        let base_area = center_rect_at_least(64, 64, 54, 16, area);
        f.render_widget(Clear, base_area);

        let popup_area = base_area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 1 });
        let block = default_block_builder("SSH Keys Management", false);
        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status
                Constraint::Min(5),    // list
                Constraint::Length(1), // message
                Constraint::Length(1), // help
            ])
            .split(inner_area);

        let status_text = format!("SSH Agent Running: {}", if self.agent_running { "Yes" } else { "No" });
        let status_style = if self.agent_running { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) };
        f.render_widget(Paragraph::new(status_text).style(status_style), chunks[0]);

        let items: Vec<ListItem> = self.keys.iter().map(|k| {
            let file_name = k.path.file_name().unwrap_or_default().to_string_lossy();
            let mut style = Style::default();
            let mut text = format!("{} ", file_name);
            
            if k.is_private {
                text.push_str("(Private) ");
            } else {
                text.push_str("(Public) ");
                style = style.fg(Color::DarkGray);
            }

            if k.is_private && !k.has_secure_permissions {
                text.push_str("[UNSECURE PERMS!]");
                style = style.fg(Color::Red).add_modifier(Modifier::BOLD);
            }

            ListItem::new(text).style(style)
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Available Keys"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[1], &mut *self.list_state.borrow_mut());

        let msg = self.message.as_deref().unwrap_or("");
        f.render_widget(Paragraph::new(msg).style(Style::default().fg(Color::Yellow)), chunks[2]);

        f.render_widget(Paragraph::new("Enter: Add | a: Start Agent | Esc: Close").style(Style::default().fg(Color::Gray)), chunks[3]);
    }
}
