use tauri::{command, State, AppHandle, Emitter};
use crate::db::DbState;
use crate::network::NetworkState;
use serde::Serialize;
use reticulum_rs::transport::packet::{Packet, Header, PacketContext, PacketDataBuffer};
use reticulum_rs::transport::hash::AddressHash;

#[derive(Serialize)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub content: Option<String>,
    pub msg_type: String,
    pub attachment_path: Option<String>,
    pub timestamp: String,
}

#[command]
pub async fn get_identity(network: State<'_, NetworkState>) -> Result<String, String> {
    Ok(network.identity.address_hash().to_hex_string())
}

#[command]
pub async fn get_messages(db: State<'_, DbState>) -> Result<Vec<Message>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, sender, content, msg_type, attachment_path, timestamp FROM messages ORDER BY timestamp DESC")
        .map_err(|e| e.to_string())?;
        
    let message_iter = stmt
        .query_map([], |row| {
            Ok(Message {
                id: row.get(0)?,
                sender: row.get(1)?,
                content: row.get(2)?,
                msg_type: row.get(3)?,
                attachment_path: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
    let mut messages = Vec::new();
    for message in message_iter {
        messages.push(message.map_err(|e| e.to_string())?);
    }
    
    Ok(messages)
}

#[derive(Serialize)]
pub struct Contact {
    pub identity_hash: String,
    pub display_name: String,
    pub status: String,
}

#[command]
pub async fn update_location(
    db: State<'_, DbState>,
    latitude: f64,
    longitude: f64,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let location = format!("{},{}", latitude, longitude);
    conn.execute(
        "INSERT OR REPLACE INTO profile (key, value) VALUES ('last_location', ?)",
        [location],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn add_contact(
    db: State<'_, DbState>,
    network: State<'_, NetworkState>,
    identity_hash: String,
    display_name: String,
) -> Result<(), String> {
    let my_id = network.identity.address_hash().to_hex_string();
    let my_name = "User".to_string(); // TODO: Get from profile

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        // 1. Insert as pending
        conn.execute(
            "INSERT OR REPLACE INTO contacts (identity_hash, display_name, status) VALUES (?, ?, ?)",
            (&identity_hash, &display_name, "pending"),
        )
        .map_err(|e| e.to_string())?;
    }
    
    // 2. Send handshake request via Reticulum
    let protocol = crate::network::Protocol::HandshakeRequest {
        sender_id: my_id,
        sender_name: my_name,
    };
    
    let payload = serde_json::to_string(&protocol).map_err(|e| e.to_string())?;
    let recipient_hash = AddressHash::new_from_hex_string(&identity_hash).map_err(|e| format!("{:?}", e))?;
    
    let packet = Packet {
        header: Header::default(),
        ifac: None,
        destination: recipient_hash,
        transport: None,
        context: PacketContext::None,
        data: PacketDataBuffer::new_from_slice(payload.as_bytes()),
    };
    
    let transport = network.transport.lock().await;
    transport.send_packet(packet).await;
    
    Ok(())
}

#[command]
pub async fn accept_handshake(
    network: State<'_, NetworkState>,
    db: State<'_, DbState>,
    identity_hash: String,
) -> Result<(), String> {
    let my_id = network.identity.address_hash().to_hex_string();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        // 1. Update status to accepted
        conn.execute(
            "UPDATE contacts SET status = 'accepted' WHERE identity_hash = ?",
            [&identity_hash],
        )
        .map_err(|e| e.to_string())?;
    }
    
    // 2. Send handshake acceptance
    let protocol = crate::network::Protocol::HandshakeAccept {
        sender_id: my_id.clone(),
    };
    
    let payload = serde_json::to_string(&protocol).map_err(|e| e.to_string())?;
    let recipient_hash = AddressHash::new_from_hex_string(&identity_hash).map_err(|e| format!("{:?}", e))?;
    
    let packet = Packet {
        header: Header::default(),
        ifac: None,
        destination: recipient_hash,
        transport: None,
        context: PacketContext::None,
        data: PacketDataBuffer::new_from_slice(payload.as_bytes()),
    };
    
    let transport = network.transport.lock().await;
    transport.send_packet(packet).await;
    
    // 3. Send last 10 messages as history sync
    let last_messages = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, content, timestamp, msg_type FROM messages ORDER BY timestamp DESC LIMIT 10").map_err(|e| e.to_string())?;
        let msg_iter = stmt.query_map([], |row| {
            Ok(crate::network::SyncMessage {
                id: row.get(0)?,
                content: row.get(1)?,
                timestamp: row.get(2)?,
                msg_type: row.get(3)?,
            })
        }).map_err(|e| e.to_string())?;
        
        let mut msgs = Vec::new();
        for msg in msg_iter {
            msgs.push(msg.map_err(|e| e.to_string())?);
        }
        msgs
    };
    
    let sync_protocol = crate::network::Protocol::HistorySync {
        sender_id: my_id,
        messages: last_messages,
    };
    
    let sync_payload = serde_json::to_string(&sync_protocol).map_err(|e| e.to_string())?;
    let sync_packet = Packet {
        header: Header::default(),
        ifac: None,
        destination: recipient_hash,
        transport: None,
        context: PacketContext::None,
        data: PacketDataBuffer::new_from_slice(sync_payload.as_bytes()),
    };
    
    transport.send_packet(sync_packet).await;
    
    Ok(())
}

#[command]
pub async fn get_contacts(db: State<'_, DbState>) -> Result<Vec<Contact>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT identity_hash, display_name, status FROM contacts WHERE status = 'accepted' ORDER BY display_name ASC")
        .map_err(|e| e.to_string())?;
        
    let contact_iter = stmt
        .query_map([], |row| {
            Ok(Contact {
                identity_hash: row.get(0)?,
                display_name: row.get(1)?,
                status: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
    let mut contacts = Vec::new();
    for contact in contact_iter {
        contacts.push(contact.map_err(|e| e.to_string())?);
    }
    
    Ok(contacts)
}

#[command]
pub async fn get_pending_contacts(db: State<'_, DbState>) -> Result<Vec<Contact>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT identity_hash, display_name, status FROM contacts WHERE status = 'pending' ORDER BY display_name ASC")
        .map_err(|e| e.to_string())?;
        
    let contact_iter = stmt
        .query_map([], |row| {
            Ok(Contact {
                identity_hash: row.get(0)?,
                display_name: row.get(1)?,
                status: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
    let mut contacts = Vec::new();
    for contact in contact_iter {
        contacts.push(contact.map_err(|e| e.to_string())?);
    }
    
    Ok(contacts)
}

#[command]
pub async fn send_message(
    app: AppHandle,
    db: State<'_, DbState>,
    network: State<'_, NetworkState>,
    content: String,
    recipient: String,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let sender = network.identity.address_hash().to_hex_string();
    
    // 1. Save to local DB
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (id, sender, content, msg_type) VALUES (?, ?, ?, ?)",
            (&id, &sender, &content, "text"),
        )
        .map_err(|e| e.to_string())?;
    }
    
    // 2. Notify via Tauri Event
    app.emit("new-message", &id).map_err(|e| e.to_string())?;
    
    // 3. Send via Reticulum using the Protocol
    let protocol = crate::network::Protocol::ChatMessage {
        sender_id: sender,
        content,
    };
    
    let payload = serde_json::to_string(&protocol).map_err(|e| e.to_string())?;
    let recipient_hash = AddressHash::new_from_hex_string(&recipient).map_err(|e| format!("{:?}", e))?;
    
    let packet = Packet {
        header: Header::default(),
        ifac: None,
        destination: recipient_hash,
        transport: None,
        context: PacketContext::None,
        data: PacketDataBuffer::new_from_slice(payload.as_bytes()),
    };
    
    let transport = network.transport.lock().await;
    transport.send_packet(packet).await;
    
    println!("Sent protocol message to {}: {:?}", recipient, protocol);
    
    Ok(())
}

#[command]
pub async fn send_image(
    app: AppHandle,
    db: State<'_, DbState>,
    network: State<'_, NetworkState>,
    image_data: Vec<u8>,
    _recipient: String,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let sender = network.identity.address_hash().to_hex_string();

    // 1. Process to AVIF
    let avif_data = crate::media::process_image_to_avif(&image_data).map_err(|e| e.to_string())?;
    
    // 2. Save locally
    let path = crate::media::save_media(&app, &avif_data, "avif").map_err(|e| e.to_string())?;
    let path_str = path.to_string_lossy().to_string();

    // 3. Save to local DB
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (id, sender, msg_type, attachment_path) VALUES (?, ?, ?, ?)",
            (&id, &sender, "image", &path_str),
        )
        .map_err(|e| e.to_string())?;
    }

    // 4. Notify via Tauri Event
    app.emit("new-message", &id).map_err(|e| e.to_string())?;

    // 5. Send via Reticulum (Placeholder for Resource API)
    // For now, we'll just log it. Reticulum Resources are needed for larger files.
    println!("Image saved and ready for transfer: {}", path_str);
    
    Ok(())
}

#[command]
pub async fn send_audio(
    app: AppHandle,
    db: State<'_, DbState>,
    network: State<'_, NetworkState>,
    pcm_data: Vec<f32>,
    _recipient: String,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let sender = network.identity.address_hash().to_hex_string();

    // 1. Process to Opus
    let opus_data = crate::media::process_audio_to_opus(&pcm_data).map_err(|e| e.to_string())?;
    
    // 2. Save locally
    let path = crate::media::save_media(&app, &opus_data, "opus").map_err(|e| e.to_string())?;
    let path_str = path.to_string_lossy().to_string();

    // 3. Save to local DB
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (id, sender, msg_type, attachment_path) VALUES (?, ?, ?, ?)",
            (&id, &sender, "audio", &path_str),
        )
        .map_err(|e| e.to_string())?;
    }

    // 4. Notify via Tauri Event
    app.emit("new-message", &id).map_err(|e| e.to_string())?;

    // 5. Send via Reticulum (Placeholder)
    println!("Audio saved and ready for transfer: {}", path_str);
    
    Ok(())
}
