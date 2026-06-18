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
use yggdrasil::core::Core;
use yggdrasil::config::Config;
use ed25519_dalek::SigningKey;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Protocol {
    HandshakeRequest {
        sender_id: String,
        sender_name: String,
    },
    HandshakeAccept {
        sender_id: String,
    },
    ChatMessage {
        sender_id: String,
        content: String,
    },
    HistorySync {
        sender_id: String,
        messages: Vec<SyncMessage>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncMessage {
    pub id: String,
    pub content: String,
    pub timestamp: String,
    pub msg_type: String,
}

#[allow(dead_code)]
pub struct NetworkState {
    pub identity: PrivateIdentity,
    pub transport: Arc<Mutex<Transport>>,
    pub address: AddressHash,
    pub ygg_core: Arc<Core>,
    pub ygg_address: String,
}

pub async fn init_network(app_handle: &AppHandle) -> Result<NetworkState, Box<dyn std::error::Error>> {
    let app_dir = app_handle.path().app_data_dir()?;
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    // --- 1. Initialize Reticulum Identity ---
    let id_path = app_dir.join("identity.key");
    
    let identity = if id_path.exists() {
        let key_data = fs::read(&id_path)?;
        PrivateIdentity::from_private_key_bytes(&key_data).unwrap_or_else(|_| PrivateIdentity::new_from_name(&uuid::Uuid::new_v4().to_string()))
    } else {
        let new_id = PrivateIdentity::new_from_name(&uuid::Uuid::new_v4().to_string());
        fs::write(&id_path, new_id.to_private_key_bytes())?;
        new_id
    };
    
    println!("Reticulum Private Identity initialized: {}", identity.address_hash());

    // --- 2. Initialize Yggdrasil ---
    let ygg_config_path = app_dir.join("yggdrasil.toml");
    let ygg_config = if ygg_config_path.exists() {
        let content = fs::read_to_string(&ygg_config_path)?;
        toml::from_str::<Config>(&content).unwrap_or_else(|_| Config::default())
    } else {
        let config = Config::default();
        let content = toml::to_string(&config)?;
        fs::write(&ygg_config_path, content)?;
        config
    };

    let priv_bytes = identity.to_private_key_bytes();
    let signing_key = SigningKey::from_bytes(&priv_bytes[32..].try_into().unwrap());
    let ygg_core = Core::new(signing_key, ygg_config);
    let ygg_address = ygg_core.address().to_string();
    println!("Yggdrasil initialized with address: {}", ygg_address);

    // --- 3. Initialize Reticulum Transport ---
    let config = TransportConfig::new("rnsd", &identity, true);
    let mut transport = Transport::new(config);
    
    // Add UDP Interface binding to all interfaces (IPv4 and IPv6)
    // This allows RNS to communicate over both local LAN and Yggdrasil mesh
    {
        let manager_arc = transport.iface_manager();
        let mut iface_manager = manager_arc.lock().await;
        
        // Listen on IPv4 and IPv6
        // IPv6
        let udp_iface_v6 = UdpInterface::new("[::]:29716", None);
        iface_manager.spawn(udp_iface_v6, UdpInterface::spawn);

        // IPv4 (Essential for some Android hotspot/local mesh scenarios)
        let udp_iface_v4 = UdpInterface::new("0.0.0.0:29716", None);
        iface_manager.spawn(udp_iface_v4, UdpInterface::spawn);
    }
    
    let dest_name = DestinationName::new("rnsd", "chat");
    let destination = transport.add_destination(
        identity.clone(),
        dest_name,
    ).await;
    
    let address = *destination.lock().await.identity.address_hash();
    let transport = Arc::new(Mutex::new(transport));

    // --- 3. Start Background Listener ---
    let handle_clone = app_handle.clone();
    let transport_clone = transport.clone();
    let my_address = address;
    
    tauri::async_runtime::spawn(async move {
        println!("Reticulum background listener started for address: {}", my_address);
        let mut rx = transport_clone.lock().await.received_data_events();
        
        while let Ok(received) = rx.recv().await {
            if received.destination == my_address {
                let data = String::from_utf8_lossy(received.data.as_slice()).to_string();
                
                println!("Received data for {}: {}", my_address, data);
                
                if let Ok(protocol) = serde_json::from_str::<Protocol>(&data) {
                    if let Some(db) = handle_clone.try_state::<DbState>() {
                        let conn = db.conn.lock().unwrap();
                        
                        match protocol {
                            Protocol::HandshakeRequest { sender_id, sender_name } => {
                                println!("Received handshake request from {} ({})", sender_name, sender_id);
                                let _ = conn.execute(
                                    "INSERT OR REPLACE INTO contacts (identity_hash, display_name, status) VALUES (?, ?, ?)",
                                    (&sender_id, &sender_name, "pending"),
                                );
                                let _ = handle_clone.emit("new-friend-request", &sender_id);
                            }
                            Protocol::HandshakeAccept { sender_id } => {
                                println!("Received handshake acceptance from {}", sender_id);
                                let _ = conn.execute(
                                    "UPDATE contacts SET status = 'accepted' WHERE identity_hash = ?",
                                    [&sender_id],
                                );
                                let _ = handle_clone.emit("friend-request-accepted", &sender_id);
                            }
                            Protocol::ChatMessage { sender_id, content } => {
                                println!("Received message from {}: {}", sender_id, content);
                                let id = uuid::Uuid::new_v4().to_string();
                                let _ = conn.execute(
                                    "INSERT INTO messages (id, sender, content, msg_type) VALUES (?, ?, ?, ?)",
                                    (&id, &sender_id, &content, "text"),
                                );
                                let _ = handle_clone.emit("new-message", &id);
                            }
                            Protocol::HistorySync { sender_id, messages } => {
                                println!("Received history sync from {} with {} messages", sender_id, messages.len());
                                for msg in messages {
                                    let _ = conn.execute(
                                        "INSERT OR IGNORE INTO messages (id, sender, content, msg_type, timestamp) VALUES (?, ?, ?, ?, ?)",
                                        (&msg.id, &sender_id, &msg.content, &msg.msg_type, &msg.timestamp),
                                    );
                                }
                                let _ = handle_clone.emit("new-message", "sync"); // Trigger full refresh
                            }
                        }
                    }
                } else {
                    // Fallback for legacy raw text messages
                    if let Some(db) = handle_clone.try_state::<DbState>() {
                        let id = uuid::Uuid::new_v4().to_string();
                        let conn = db.conn.lock().unwrap();
                        let sender = "unknown".to_string();
                        let _ = conn.execute(
                            "INSERT INTO messages (id, sender, content, msg_type) VALUES (?, ?, ?, ?)",
                            (&id, &sender, &data, "text"),
                        );
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
        ygg_core,
        ygg_address,
    })
}
