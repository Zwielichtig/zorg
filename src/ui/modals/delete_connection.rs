use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};
use crate::ui::utils::{center_rect_at_least, default_block_builder};

pub struct DeleteConnectionModal {
    pub is_open: bool,
    pub connection_id: Option<i32>,
    pub connection_name: String,
    pub selected_yes: bool,
}

impl Default for DeleteConnectionModal {
    fn default() -> Self {
        Self {
            is_open: false,
            connection_id: None,
            connection_name: String::new(),
            selected_yes: false,
        }
    }
}

impl DeleteConnectionModal {
    pub fn open(&mut self, id: i32, name: String) {
        self.is_open = true;
        self.connection_id = Some(id);
        self.connection_name = name;
        self.selected_yes = false; // default to no
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.connection_id = None;
        self.connection_name.clear();
        self.selected_yes = false;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        let base_area = center_rect_at_least(44, 50, 44, 12, area);
        f.render_widget(Clear, base_area);

        let popup_area = base_area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 1 });
        let block = default_block_builder("Confirm Deletion", false);
        f.render_widget(block.clone(), popup_area);

        let inner_area = block.inner(popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(2),
                Constraint::Length(3), // buttons
            ])
            .split(inner_area);

        let text = vec![
            Line::from(Span::styled("Are you sure you want", Style::default().fg(Color::White))),
            Line::from(Span::styled("to delete this connection?", Style::default().fg(Color::White))),
            Line::from(Span::styled(format!("({})", self.connection_name), Style::default().fg(Color::Yellow))),
        ];
        
        let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, chunks[0]);

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);

        let yes_style = if self.selected_yes {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };
        let no_style = if !self.selected_yes {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let yes_btn = Paragraph::new("[ Yes ]").alignment(ratatui::layout::Alignment::Center).style(yes_style);
        let no_btn = Paragraph::new("[ No ]").alignment(ratatui::layout::Alignment::Center).style(no_style);

        f.render_widget(yes_btn, button_chunks[0]);
        f.render_widget(no_btn, button_chunks[1]);
    }
}
