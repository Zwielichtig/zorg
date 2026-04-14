use crate::db::connection::{Connection, NewConnection};
use crate::db::history::History;
use crate::ui::modals::create_connection::CreateConnectionModal;
use diesel::SqliteConnection;

#[derive(PartialEq)]
pub enum AppFocus {
    Search,
    List,
}

pub struct App {
    pub input: String,
    pub messages: Vec<String>,
    pub create_connection_modal: CreateConnectionModal,
    pub keys_modal: crate::ui::modals::keys::KeysModal,
    pub db: SqliteConnection,
    pub connections: Vec<Connection>,
    pub pending_ssh_connection: Option<Connection>,
    pub selected_connection_index: usize,
    pub show_help_modal: bool,
    pub recent_history: Vec<History>,
    pub focus: AppFocus,
}

impl App {
    pub fn new(mut db: SqliteConnection) -> Self {
        let connections = Connection::get_all(&mut db).unwrap_or_default();
        let recent_history = History::get_recent(&mut db, 10).unwrap_or_default();
        Self {
            input: String::new(),
            messages: Vec::new(),
            create_connection_modal: CreateConnectionModal::default(),
            keys_modal: crate::ui::modals::keys::KeysModal::default(),
            db,
            connections,
            pending_ssh_connection: None,
            selected_connection_index: 0,
            show_help_modal: false,
            recent_history,
            focus: AppFocus::Search,
        }
    }

    pub fn db_conn(&mut self) -> &mut SqliteConnection {
        &mut self.db
    }

    pub fn refresh_history(&mut self) {
        if let Ok(hist) = History::get_recent(&mut self.db, 10) {
            self.recent_history = hist;
        }
    }

    pub fn refresh_connections(&mut self) {
        if let Ok(conns) = Connection::get_all(&mut self.db) {
            self.connections = conns;
        }
    }

    pub fn submit_connection(&mut self) {

        let port = if self.create_connection_modal.port.is_empty() {
            None
        } else {
            self.create_connection_modal.port.parse::<i32>().ok()
        };

        let identity_file = if self.create_connection_modal.identity_file.is_empty() {
            None
        } else {
            Some(self.create_connection_modal.identity_file.as_str())
        };

        let note = if self.create_connection_modal.note.is_empty() {
            None
        } else {
            Some(self.create_connection_modal.note.as_str())
        };

        if let Some(id) = self.create_connection_modal.editing_connection_id {
            match crate::db::connection::UpdateConnection::update(
                &mut self.db,
                id,
                &self.create_connection_modal.name,
                &self.create_connection_modal.username,
                &self.create_connection_modal.hostname,
                port,
                identity_file,
                note,
            ) {
                Ok(conn) => {
                    self.messages.push(format!("Updated connection: {}", conn.name));
                    self.refresh_connections();
                }
                Err(e) => {
                    self.messages.push(format!("Error updating connection: {}", e));
                }
            }
        } else {
            match NewConnection::create(
                &mut self.db,
                &self.create_connection_modal.name,
                &self.create_connection_modal.username,
                &self.create_connection_modal.hostname,
                port,
                identity_file,
                note,
            ) {
                Ok(conn) => {
                    self.messages.push(format!("Created connection: {}", conn.name));
                    self.refresh_connections();
                }
                Err(e) => {
                    self.messages.push(format!("Error creating connection: {}", e));
                }
            }
        }
        self.close_modal();
    }

    pub fn close_modal(&mut self) {
        self.create_connection_modal.is_open = false;
        self.create_connection_modal.reset();
    }
}
