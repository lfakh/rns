mod db;
mod network;
mod commands;
mod media;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_barcode_scanner::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_geolocation::init())
    .invoke_handler(tauri::generate_handler![
      commands::get_identity,
      commands::get_messages,
      commands::get_contacts,
      commands::add_contact,
      commands::update_location,
      commands::send_message,
      commands::send_image,
      commands::send_audio,
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
