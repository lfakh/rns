use std::fs;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Emitter};
use reticulum_rs::transport::identity::PrivateIdentity;
use reticulum_rs::runtime::{Transport, TransportConfig};
use reticulum_rs::transport::destination::DestinationName;
use reticulum_rs::iface::udp::UdpInterface;
use reticulum_rs::transport::hash::AddressHash;
use tokio::sync::Mutex;
use crate::db::DbState;

pub struct NetworkState {
    pub identity: PrivateIdentity,
    pub transport: Arc<Mutex<Transport>>,
    pub address: AddressHash,
}

pub async fn init_network(app_handle: &AppHandle) -> Result<NetworkState, Box<dyn std::error::Error>> {
    let app_dir = app_handle.path().app_data_dir()?;
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    let id_path = app_dir.join("identity.key");
    
    let identity = if id_path.exists() {
        let key_data = fs::read(&id_path)?;
        PrivateIdentity::from_private_key_bytes(&key_data).unwrap_or_else(|_| PrivateIdentity::new_from_name("bestra-default"))
    } else {
        let new_id = PrivateIdentity::new_from_name("bestra-default");
        fs::write(&id_path, new_id.to_private_key_bytes())?;
        new_id
    };
    
    println!("Reticulum Private Identity initialized: {}", identity.address_hash());

    // 1. Initialize Transport
    let config = TransportConfig::new("rnsd", &identity, true);
    let mut transport = Transport::new(config);
    
    // 2. Add UDP Interface for local discovery (Standard RNS port)
    {
        let manager_arc = transport.iface_manager();
        let mut iface_manager = manager_arc.lock().await;
        let udp_iface = UdpInterface::new("0.0.0.0:29716", None);
        iface_manager.spawn(udp_iface, UdpInterface::spawn);
    }
    
    // 3. Create Destination for this app
    let dest_name = DestinationName::new("rnsd", "chat");
    let destination = transport.add_destination(
        identity.clone(),
        dest_name,
    ).await;
    
    let address = *destination.lock().await.identity.address_hash();
    let transport = Arc::new(Mutex::new(transport));

    // 4. Start Background Listener
    let handle_clone = app_handle.clone();
    let transport_clone = transport.clone();
    let my_address = address;
    
    tauri::async_runtime::spawn(async move {
        println!("Reticulum background listener started for address: {}", my_address);
        let mut rx = transport_clone.lock().await.received_data_events();
        
        while let Ok(received) = rx.recv().await {
            if received.destination == my_address {
                let content = String::from_utf8_lossy(received.data.as_slice()).to_string();
                let sender = "unknown".to_string(); // In a real app, we'd extract the sender identity
                
                println!("Received message for {}: {}", my_address, content);
                
                // Save to DB
                if let Some(db) = handle_clone.try_state::<DbState>() {
                    let id = uuid::Uuid::new_v4().to_string();
                    let conn = db.conn.lock().unwrap();
                    let res = conn.execute(
                        "INSERT INTO messages (id, sender, content) VALUES (?, ?, ?)",
                        (&id, &sender, &content),
                    );
                    if let Err(e) = res {
                        eprintln!("Failed to save received message: {}", e);
                    } else {
                        // Notify UI
                        let _ = handle_clone.emit("new-message", id);
                    }
                }
            }
        }
    });

    Ok(NetworkState { 
        identity,
        transport,
        address,
    })
}
