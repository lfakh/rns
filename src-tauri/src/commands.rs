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
    pub content: String,
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
        .prepare("SELECT id, sender, content, timestamp FROM messages ORDER BY timestamp DESC")
        .map_err(|e| e.to_string())?;
        
    let message_iter = stmt
        .query_map([], |row| {
            Ok(Message {
                id: row.get(0)?,
                sender: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
    let mut messages = Vec::new();
    for message in message_iter {
        messages.push(message.map_err(|e| e.to_string())?);
    }
    
    Ok(messages)
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
            "INSERT INTO messages (id, sender, content) VALUES (?, ?, ?)",
            (&id, &sender, &content),
        )
        .map_err(|e| e.to_string())?;
    }
    
    // 2. Notify via Tauri Event
    app.emit("new-message", &id).map_err(|e| e.to_string())?;
    
    // 3. Send via Reticulum
    let recipient_hash = AddressHash::new_from_hex_string(&recipient).map_err(|e| format!("{:?}", e))?;
    
    let packet = Packet {
        header: Header::default(),
        ifac: None,
        destination: recipient_hash,
        transport: None,
        context: PacketContext::None,
        data: PacketDataBuffer::new_from_slice(content.as_bytes()),
    };
    
    let transport = network.transport.lock().await;
    transport.send_packet(packet).await;
    
    println!("Sent message to {}: {}", recipient, content);
    
    Ok(())
}
