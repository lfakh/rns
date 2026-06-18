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
            content TEXT,
            msg_type TEXT DEFAULT 'text',
            attachment_path TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            identity_hash TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            status TEXT DEFAULT 'accepted',
            last_sync_timestamp DATETIME
        )",
        [],
    )?;

    // Add columns if they don't exist (for existing databases)
    let _ = conn.execute("ALTER TABLE contacts ADD COLUMN status TEXT DEFAULT 'accepted'", []);
    let _ = conn.execute("ALTER TABLE contacts ADD COLUMN last_sync_timestamp DATETIME", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS profile (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(DbState { conn: Mutex::new(conn) })
}
