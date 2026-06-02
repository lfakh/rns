mod db;
mod network;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
use tauri::Manager;

pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      commands::get_identity,
      commands::get_messages,
      commands::send_message,
    ])
    .setup(|app| {
      let handle = app.handle().clone();
      tauri::async_runtime::spawn(async move {
        // Initialize DB (now synchronous)
        let db_state = match db::init_db(&handle) {
          Ok(state) => {
            handle.manage(state);
            println!("Database initialized.");
            true
          }
          Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            false
          }
        };

        // Initialize Network (now synchronous)
        let network_state = match network::init_network(&handle).await {
          Ok(state) => {
            handle.manage(state);
            println!("Reticulum initialized.");
            true
          }
          Err(e) => {
            eprintln!("Failed to initialize reticulum: {}", e);
            false
          }
        };

        if db_state && network_state {
          println!("Application core services started.");
        }
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
