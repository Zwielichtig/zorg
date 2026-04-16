use crate::db::connection::Connection;
use crate::db::history::History;
use chrono::Utc;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::process::Command;

pub fn execute_ssh_connection(
    conn: &Connection,
    db: &mut diesel::SqliteConnection,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), std::io::Error> {
    // 1. temporarily yield terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
        crossterm::cursor::Show
    )?;

    // record start time
    let start_time = Utc::now().timestamp() as i32;

    // 2. build ssh command
    let mut cmd = Command::new("ssh");

    let proxy_jumps = crate::db::hop::ConnectionHop::get_jumps(db, conn.id.unwrap_or(0))
        .unwrap_or_default();

    if !proxy_jumps.is_empty() {
        let jump_strings: Vec<String> = proxy_jumps.into_iter().map(|jump| {
            let port_suffix = jump.port.map(|p| format!(":{}", p)).unwrap_or_else(|| "".to_string());
            format!("{}@{}{}", jump.username, jump.hostname, port_suffix)
        }).collect();
        cmd.arg("-J").arg(jump_strings.join(","));
    }

    if let Some(port) = conn.port {
        cmd.arg("-p").arg(port.to_string());
    }

    if let Some(ref identity) = conn.identity_file {
        cmd.arg("-i").arg(identity);
    }

    let user_host = format!("{}@{}", conn.username, conn.hostname);
    cmd.arg(user_host);

    // 3. execute and wait - the child process inherits stdin/out/err
    let status_result = cmd.status();

    if let Ok(status) = &status_result {
        // 4. record history
        let end_time = Utc::now().timestamp() as i32;
        let exit_code = if status.success() {
            "0".to_string()
        } else {
            match status.code() {
                Some(code) => code.to_string(),
                None => "unknown".to_string(),
            }
        };

        if let Some(id) = conn.id {
            let _ = History::create(db, id, start_time, end_time, exit_code);
        }
    }

    // 5. restore terminal
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        crossterm::cursor::Hide
    );
    let _ = terminal.clear();

    // return error from status if there was one
    status_result.map(|_| ())
}
