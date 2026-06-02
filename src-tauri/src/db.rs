use std::fs;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;
use rusqlite::Connection;

pub struct DbState {
    pub conn: Mutex<Connection>,
}

pub fn init_db(app_handle: &AppHandle) -> Result<DbState, Box<dyn std::error::Error>> {
    let app_dir = app_handle.path().app_data_dir()?;
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    let db_path = app_dir.join("rnsd.db");
    let conn = Connection::open(db_path)?;
    
    // Run migrations
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            sender TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            identity_hash TEXT PRIMARY KEY,
            display_name TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(DbState { conn: Mutex::new(conn) })
}
