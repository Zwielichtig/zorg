mod app;
mod db;
mod ui;
mod input;
mod ssh;

use app::App;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use crossterm::event::{self, Event};
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "zorg",
    version,
    about = "A TUI SSH connection manager"
)]
struct Cli {}


fn main() -> Result<(), io::Error> {
    let _cli = Cli::parse();

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let db = db::establish_connection();
    let mut app = App::new(db);

    loop {
        terminal.draw(|f| {
            ui::draw(f, &app);
        })?;

        let app_event = event::read()?;
        match app_event {
            Event::Key(key) => {
                if input::handle_key_event(&mut app, key) {
                    break;
                }
            }
            Event::Mouse(mouse_event) => {
                if input::handle_mouse_event(&mut app, mouse_event) {
                    break;
                }
            }
            _ => {}
        }
        
        // check if there is a pending SSH connection to run outside the TUI loop
        if let Some(conn) = app.pending_ssh_connection.take() {
            let mut db_conn = app.db_conn();
            if let Err(e) = ssh::runner::execute_ssh_connection(&conn, &mut db_conn, &mut terminal) {
                app.messages.push(format!("SSH Error: {}", e));
            }
            app.refresh_history();
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
