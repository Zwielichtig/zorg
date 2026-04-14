use crate::db::connection::{Connection, NewConnection};
use crate::db::history::History;
use crate::ui::modals::create_connection::CreateConnectionModal;
use crate::ui::modals::delete_connection::DeleteConnectionModal;
use diesel::SqliteConnection;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

#[derive(Clone, Default)]
pub struct FuzzyMatchResult {
    pub conn_index: usize,
    pub score: i64,
    pub name_indices: Vec<usize>,
    pub username_indices: Vec<usize>,
    pub hostname_indices: Vec<usize>,
}

#[derive(PartialEq)]
pub enum AppFocus {
    Search,
    List,
}

pub struct App {
    pub input: String,
    pub messages: Vec<String>,
    pub create_connection_modal: CreateConnectionModal,
    pub delete_connection_modal: DeleteConnectionModal,
    pub keys_modal: crate::ui::modals::keys::KeysModal,
    pub db: SqliteConnection,
    pub connections: Vec<Connection>,
    pub filtered_connections: Vec<FuzzyMatchResult>,
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
        let mut app = Self {
            input: String::new(),
            messages: Vec::new(),
            create_connection_modal: CreateConnectionModal::default(),
            delete_connection_modal: DeleteConnectionModal::default(),
            keys_modal: crate::ui::modals::keys::KeysModal::default(),
            db,
            connections,
            filtered_connections: Vec::new(),
            pending_ssh_connection: None,
            selected_connection_index: 0,
            show_help_modal: false,
            recent_history,
            focus: AppFocus::Search,
        };
        app.update_search_filter();
        app
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
            self.update_search_filter();
        }
    }

    pub fn update_search_filter(&mut self) {
        let matcher = SkimMatcherV2::default();
        let query = &self.input;
        
        let mut results = Vec::new();
        
        if query.is_empty() {
            for (index, _) in self.connections.iter().enumerate() {
                results.push(FuzzyMatchResult {
                    conn_index: index,
                    score: Default::default(),
                    name_indices: Vec::new(),
                    username_indices: Vec::new(),
                    hostname_indices: Vec::new(),
                });
            }
        } else {
            for (index, conn) in self.connections.iter().enumerate() {
                let combined = format!("{} {} {}", conn.name, conn.username, conn.hostname);
                if let Some((score, indices)) = matcher.fuzzy_indices(&combined, query) {
                    let name_len = conn.name.chars().count();
                    let user_len = conn.username.chars().count();
                    
                    let name_start = 0;
                    let name_end = name_len;
                    let user_start = name_end + 1;
                    let user_end = user_start + user_len;
                    let host_start = user_end + 1;
                    
                    let mut name_idx = Vec::new();
                    let mut user_idx = Vec::new();
                    let mut host_idx = Vec::new();
                    
                    for idx in indices {
                        if idx < name_end {
                            name_idx.push(idx);
                        } else if idx >= user_start && idx < user_end {
                            user_idx.push(idx - user_start);
                        } else if idx >= host_start {
                            host_idx.push(idx - host_start);
                        }
                    }
                    results.push(FuzzyMatchResult {
                        conn_index: index,
                        score,
                        name_indices: name_idx,
                        username_indices: user_idx,
                        hostname_indices: host_idx,
                    });
                }
            }
            results.sort_by(|a, b| b.score.cmp(&a.score));
        }
        
        self.filtered_connections = results;
        if self.selected_connection_index >= self.filtered_connections.len() && !self.filtered_connections.is_empty() {
            self.selected_connection_index = self.filtered_connections.len() - 1;
        } else if self.filtered_connections.is_empty() {
            self.selected_connection_index = 0;
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

    pub fn delete_connection(&mut self) {
        if let Some(id) = self.delete_connection_modal.connection_id {
            match crate::db::connection::Connection::delete(&mut self.db, id) {
                Ok(_) => {
                    self.messages.push(format!("Deleted connection: {}", self.delete_connection_modal.connection_name));
                    self.refresh_connections();
                    if self.selected_connection_index >= self.connections.len() && !self.connections.is_empty() {
                        self.selected_connection_index = self.connections.len() - 1;
                    } else if self.connections.is_empty() {
                        self.selected_connection_index = 0;
                    }
                }
                Err(e) => {
                    self.messages.push(format!("Error deleting connection: {}", e));
                }
            }
        }
        self.delete_connection_modal.close();
    }
}
