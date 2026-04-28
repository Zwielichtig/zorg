use crate::db::connection::{Connection, NewConnection};
use crate::db::history::History;
use crate::ui::modals::create_connection::CreateConnectionModal;
use crate::ui::modals::delete_connection::DeleteConnectionModal;
use crate::ui::modals::proxy_jumps::ProxyJumpsModal;
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
    pub input_cursor: usize,
    pub messages: Vec<String>,
    pub create_connection_modal: CreateConnectionModal,
    pub delete_connection_modal: DeleteConnectionModal,
    pub keys_modal: crate::ui::modals::keys::KeysModal,
    pub proxy_jumps_modal: ProxyJumpsModal,
    pub proxy_hosts: std::collections::HashSet<i32>,        // target_connection_id
    pub proxy_destinations: std::collections::HashSet<i32>, // source_connection_id
    pub db: SqliteConnection,
    pub connections: Vec<Connection>,
    pub filtered_connections: Vec<FuzzyMatchResult>,
    pub pending_ssh_connection: Option<Connection>,
    pub selected_connection_index: usize,
    pub show_help_modal: bool,
    pub recent_history: Vec<History>,
    pub history_scroll: usize,
    pub connection_history: Vec<History>,
    pub connection_history_scroll: usize,
    pub focus: AppFocus,
}

impl App {
    pub fn new(mut db: SqliteConnection) -> Self {
        let recent_history = History::get_recent(&mut db, 50).unwrap_or_default();
        let mut app = Self {
            input: String::new(),
            input_cursor: 0,
            messages: Vec::new(),
            create_connection_modal: CreateConnectionModal::default(),
            delete_connection_modal: DeleteConnectionModal::default(),
            keys_modal: crate::ui::modals::keys::KeysModal::default(),
            proxy_jumps_modal: ProxyJumpsModal::default(),
            proxy_hosts: std::collections::HashSet::new(),
            proxy_destinations: std::collections::HashSet::new(),
            db,
            connections: Vec::new(),
            filtered_connections: Vec::new(),
            pending_ssh_connection: None,
            selected_connection_index: 0,
            show_help_modal: false,
            recent_history,
            history_scroll: 0,
            connection_history: Vec::new(),
            connection_history_scroll: 0,
            focus: AppFocus::Search,
        };
        app.refresh_connections();
        app
    }

    pub fn db_conn(&mut self) -> &mut SqliteConnection {
        &mut self.db
    }

    pub fn refresh_history(&mut self) {
        if let Ok(hist) = History::get_recent(&mut self.db, 50) {
            self.recent_history = hist;
        }
        // Clamp scroll in case the new history is shorter
        self.clamp_history_scroll();
    }

    pub fn scroll_history_up(&mut self, step: usize) {
        self.history_scroll = self.history_scroll.saturating_sub(step);
    }

    pub fn scroll_history_down(&mut self, step: usize) {
        let max = self.recent_history.len().saturating_sub(1);
        self.history_scroll = (self.history_scroll + step).min(max);
    }

    fn clamp_history_scroll(&mut self) {
        let max = self.recent_history.len().saturating_sub(1);
        self.history_scroll = self.history_scroll.min(max);
    }

    pub fn refresh_connection_history(&mut self) {
        if self.filtered_connections.is_empty()
            || self.selected_connection_index >= self.filtered_connections.len()
        {
            self.connection_history = Vec::new();
            self.connection_history_scroll = 0;
            return;
        }

        let conn_idx = self.filtered_connections[self.selected_connection_index].conn_index;
        let conn_id = self.connections[conn_idx].id;

        if let Some(id) = conn_id {
            if let Ok(hist) = History::get_by_connection(&mut self.db, id, 50) {
                self.connection_history = hist;
            }
        } else {
            self.connection_history = Vec::new();
        }
        self.clamp_connection_history_scroll();
    }

    pub fn scroll_connection_history_up(&mut self, step: usize) {
        self.connection_history_scroll = self.connection_history_scroll.saturating_sub(step);
    }

    pub fn scroll_connection_history_down(&mut self, step: usize) {
        let max = self.connection_history.len().saturating_sub(1);
        self.connection_history_scroll = (self.connection_history_scroll + step).min(max);
    }

    fn clamp_connection_history_scroll(&mut self) {
        let max = self.connection_history.len().saturating_sub(1);
        self.connection_history_scroll = self.connection_history_scroll.min(max);
    }

    pub fn refresh_connections(&mut self) {
        if let Ok(conns) = Connection::get_all(&mut self.db) {
            self.connections = conns;
            // proxy hosts (used as jump targets)
            if let Ok(hosts) =
                crate::db::hop::ConnectionHop::get_all_jump_target_ids(&mut self.db)
            {
                self.proxy_hosts = hosts;
            } else {
                self.proxy_hosts.clear();
            }

            // proxy destinations (use a proxy)
            if let Ok(destinations) =
                crate::db::hop::ConnectionHop::get_all_proxy_destination_ids(&mut self.db)
            {
                self.proxy_destinations = destinations;
            } else {
                self.proxy_destinations.clear();
            }
            self.update_search_filter();
        }
    }

    pub fn has_proxy(&self, connection: &Connection) -> bool {
        connection
            .id
            .is_some_and(|id| self.proxy_destinations.contains(&id))
    }

    pub fn is_proxy(&self, connection: &Connection) -> bool {
        connection
            .id
            .is_some_and(|id| self.proxy_hosts.contains(&id))
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
        }
        
        results.sort_by(|a, b| {
            let conn_a = &self.connections[a.conn_index];
            let conn_b = &self.connections[b.conn_index];

            let a_score = a.score;
            let b_score = b.score;

            // metadata (computed once per comparison)
            let a_has_proxy = self.has_proxy(conn_a);
            let a_is_proxy = self.is_proxy(conn_a);

            let b_has_proxy = self.has_proxy(conn_b);
            let b_is_proxy = self.is_proxy(conn_b);

            // convert classification into stable priority tiers
            let a_priority: u8 = match (a_has_proxy, a_is_proxy) {
                (false, false) => 0, // default
                (true, false)  => 1, // has_proxy
                (false, true)  => 2, // is_proxy
                (true, true)   => 3, // both
            };

            let b_priority: u8 = match (b_has_proxy, b_is_proxy) {
                (false, false) => 0,
                (true, false)  => 1,
                (false, true)  => 2,
                (true, true)   => 3,
            };

            // PRIMARY: fuzzy score (higher is better)
            b_score
                .cmp(&a_score)
                // SECONDARY: proxy classification (lower tier first)
                .then_with(|| a_priority.cmp(&b_priority))
        });
        
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
