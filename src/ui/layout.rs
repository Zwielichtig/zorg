use crate::app::App;
use crate::ui::components::{
    config_menu::render_config_menu, history::render_history, search_input::render_search_input,
    connection_list::render_connection_list,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Clear, Paragraph},
    Frame,
};
use crate::ui::utils::center_rect;

pub fn draw(f: &mut Frame, app: &App) {
    let dimmed = app.create_connection_modal.is_open || app.delete_connection_modal.is_open || app.show_help_modal;
    let style = if dimmed {
        Style::default().fg(Color::Indexed(244))
    } else {
        Style::default()
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_search_input(f, app, left_chunks[0], dimmed, style);
    render_connection_list(f, app, left_chunks[1], dimmed, style);
    render_history(f, right_chunks[0], dimmed);
    render_config_menu(f, right_chunks[1], dimmed);

    let shortcuts = " [Enter] Connect | [f] Toggle Favorite | [^n] New | [^e] Edit | [^d] Delete | [^p] Proxyjump | [↑/↓] Navigate | [^h] Help";
    f.render_widget(
        Paragraph::new(shortcuts).style(Style::default().fg(Color::Yellow)),
        main_chunks[1],
    );

    if app.show_help_modal {
        let base_area = center_rect(44, 44, f.area());
        f.render_widget(Clear, base_area);
        let help_area = base_area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 1 });
        let help_text = "== Zorg Shortcuts ==\n\n\
            [Enter]      Connect selected\n\
            [Tab]        Navigate blocks\n\
            [Ctrl+n]     Create new connection\n\
            [Ctrl+e]     Edit selected connection\n\
            [Ctrl+d]     Delete selected connection\n\
            [Ctrl+p]     Manage Proxyjumps\n\
            [f]       Toggle favorite status\n\
            [↑/↓]     Navigate list\n\
            [Ctrl+h]     Toggle this help modal\n\
            [Ctrl+c]     Quit application\n\
            [Esc]     Close modal / Quit application";
        let help_block = crate::ui::utils::default_block_builder("Help (Press Esc to close)", false);
        f.render_widget(Paragraph::new(help_text).block(help_block), help_area);
    }
}
