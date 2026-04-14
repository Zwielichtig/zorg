pub mod connection;
pub mod schema;
pub mod history;

use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::fs;
use std::path::Path;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let default_db = env::var("HOME")
        .map(|home| format!("{}/.local/share/zorg/zorg.db", home))
        .unwrap_or_else(|_| "zorg.db".to_string());

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| default_db);
    
    // Ensure the database parent directory exists
    if let Some(parent) = Path::new(&database_url).parent() {
        if !parent.exists() && parent != Path::new("") {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                panic!("Failed to create database directory {:?}: {}", parent, e);
            });
        }
    }

    let mut conn = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    conn.run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|e| panic!("Failed to run migrations: {}", e));

    conn
}
