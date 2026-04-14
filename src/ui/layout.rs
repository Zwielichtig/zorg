use crate::app::App;
use crate::ui::components::{
    config_menu::render_config_menu, history::render_history, search_input::render_search_input,
    connection_list::render_search_results,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Clear, Paragraph},
    Frame,
};
use crate::ui::utils::center_rect;

pub fn draw(f: &mut Frame, app: &App) {
    let dimmed = app.create_connection_modal.is_open || app.show_help_modal;
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
        .constraints([Constraint::Percentage(5), Constraint::Percentage(95)])
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_search_input(f, app, left_chunks[0], dimmed, style);
    render_search_results(f, app, left_chunks[1], dimmed);
    render_history(f, app, right_chunks[0], dimmed, style);
    render_config_menu(f, right_chunks[1], dimmed);

    let shortcuts = " [Enter] Connect  |  [f] Toggle Favorite  |  [Ctrl+n] New  |  [Ctrl+e] Edit  |  [Ctrl+k] Keys  |  [Up/Down] Navigate  |  [?] Full Help";
    f.render_widget(
        Paragraph::new(shortcuts).style(Style::default().fg(Color::Yellow)),
        main_chunks[1],
    );

    if app.show_help_modal {
        let help_area = center_rect(40, 40, f.area());
        f.render_widget(Clear, help_area);
        let help_text = "== Zorg Shortcuts ==\n\n\
            [Enter]   Connect selected\n\
            [f]       Toggle favorite status\n\
            [Ctrl+n]  Create new connection\n\
            [Ctrl+e]  Edit selected connection\n\
            [Ctrl+k]  Manage SSH Keys\n\
            [Up/Down] Navigate list\n\
            [?]       Toggle this help modal\n\
            [Ctrl+c]  Quit application\n\
            [Esc]     Go back / Close modal";
        let help_block = crate::ui::utils::default_block_builder("Help (Press Esc to close)", false);
        f.render_widget(Paragraph::new(help_text).block(help_block), help_area);
    }
}
