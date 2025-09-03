//! OpenFire communication module for XMPP messaging

use crate::auth::{AuthManager, Credentials};
use crate::config::Config;
use crate::error::{OpenFireError, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// XMPP message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Chat,
    GroupChat,
    Headline,
    Normal,
    Error,
}

/// XMPP message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub message_type: MessageType,
    pub subject: Option<String>,
    pub body: String,
    pub timestamp: u64,
    pub thread: Option<String>,
}

impl Message {
    pub fn new_chat(from: String, to: String, body: String) -> Self {
        Self {
            id: generate_message_id(),
            from,
            to,
            message_type: MessageType::Chat,
            subject: None,
            body,
            timestamp: current_timestamp(),
            thread: None,
        }
    }
    
    pub fn new_group_chat(from: String, to: String, body: String) -> Self {
        Self {
            id: generate_message_id(),
            from,
            to,
            message_type: MessageType::GroupChat,
            subject: None,
            body,
            timestamp: current_timestamp(),
            thread: None,
        }
    }
}

/// Presence status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresenceStatus {
    Available,
    Away,
    ExtendedAway,
    DoNotDisturb,
    Unavailable,
    Invisible,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    pub jid: String,
    pub status: PresenceStatus,
    pub status_message: Option<String>,
    pub priority: i8,
    pub timestamp: u64,
}

impl Presence {
    pub fn new(jid: String, status: PresenceStatus) -> Self {
        Self {
            jid,
            status,
            status_message: None,
            priority: 0,
            timestamp: current_timestamp(),
        }
    }
    
    pub fn with_message(mut self, message: String) -> Self {
        self.status_message = Some(message);
        self
    }
    
    pub fn with_priority(mut self, priority: i8) -> Self {
        self.priority = priority;
        self
    }
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub jid: String,
    pub name: Option<String>,
    pub subscription: String,
    pub groups: Vec<String>,
    pub presence: Option<Presence>,
}

/// Chat room information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub jid: String,
    pub name: String,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub participants: Vec<String>,
    pub joined: bool,
}

/// Event types for the message callback
#[derive(Debug, Clone)]
pub enum XmppEvent {
    MessageReceived(Message),
    PresenceUpdated(Presence),
    ContactUpdated(Contact),
    ConnectionStateChanged(String),
    Error(String),
}

/// Message callback type
pub type EventCallback = Box<dyn Fn(XmppEvent) + Send + Sync>;

/// OpenFire XMPP client
pub struct OpenFireClient {
    config: Config,
    auth_manager: Arc<Mutex<AuthManager>>,
    is_connected: bool,
    current_presence: Option<Presence>,
    contacts: HashMap<String, Contact>,
    chat_rooms: HashMap<String, ChatRoom>,
    event_tx: Option<mpsc::UnboundedSender<XmppEvent>>,
}

impl OpenFireClient {
    /// Create a new OpenFire client
    pub fn new(config: Config) -> Result<Self> {
        let auth_manager = Arc::new(Mutex::new(AuthManager::new(config.clone())?));
        
        Ok(Self {
            config,
            auth_manager,
            is_connected: false,
            current_presence: None,
            contacts: HashMap::new(),
            chat_rooms: HashMap::new(),
            event_tx: None,
        })
    }
    
    /// Connect and authenticate to the OpenFire server
    pub async fn connect(&mut self, credentials: Credentials) -> Result<()> {
        info!("Connecting to OpenFire server: {}", self.config.server);
        
        let mut auth_manager = self.auth_manager.lock().await;
        let auth_result = auth_manager.authenticate(credentials).await?;
        
        if !auth_result.success {
            return Err(OpenFireError::AuthenticationFailed {
                message: auth_result.message,
            });
        }
        
        self.is_connected = true;
        
        // Set initial presence
        let jid = auth_result.full_jid.unwrap_or_else(|| "unknown@localhost".to_string());
        self.current_presence = Some(Presence::new(jid, PresenceStatus::Available));
        
        info!("Successfully connected to OpenFire server");
        self.emit_event(XmppEvent::ConnectionStateChanged("connected".to_string())).await;
        
        Ok(())
    }
    
    /// Disconnect from the OpenFire server
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from OpenFire server");
        
        let mut auth_manager = self.auth_manager.lock().await;
        auth_manager.disconnect().await?;
        
        self.is_connected = false;
        self.current_presence = None;
        
        info!("Disconnected from OpenFire server");
        self.emit_event(XmppEvent::ConnectionStateChanged("disconnected".to_string())).await;
        
        Ok(())
    }
    
    /// Check if connected to the server
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
    
    /// Send a chat message
    pub async fn send_message(&self, to: &str, body: &str) -> Result<String> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        let from = self.current_presence
            .as_ref()
            .map(|p| p.jid.clone())
            .unwrap_or_else(|| "unknown@localhost".to_string());
            
        let message = Message::new_chat(from, to.to_string(), body.to_string());
        
        info!("Sending message to {}: {}", to, body);
        
        // In a real implementation, this would send the message via XMPP
        // For now, we'll simulate the sending process
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        debug!("Message sent successfully: {}", message.id);
        
        Ok(message.id)
    }
    
    /// Send a group chat message
    pub async fn send_group_message(&self, room_jid: &str, body: &str) -> Result<String> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        if !self.chat_rooms.contains_key(room_jid) {
            return Err(OpenFireError::XmppProtocolError {
                message: format!("Not joined to room: {}", room_jid),
            });
        }
        
        let from = self.current_presence
            .as_ref()
            .map(|p| p.jid.clone())
            .unwrap_or_else(|| "unknown@localhost".to_string());
            
        let message = Message::new_group_chat(from, room_jid.to_string(), body.to_string());
        
        info!("Sending group message to {}: {}", room_jid, body);
        
        // Simulate sending group message
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        debug!("Group message sent successfully: {}", message.id);
        
        Ok(message.id)
    }
    
    /// Update presence status
    pub async fn set_presence(&mut self, status: PresenceStatus, message: Option<String>) -> Result<()> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        let jid = self.current_presence
            .as_ref()
            .map(|p| p.jid.clone())
            .unwrap_or_else(|| "unknown@localhost".to_string());
            
        let mut presence = Presence::new(jid, status);
        if let Some(msg) = message {
            presence = presence.with_message(msg);
        }
        
        info!("Setting presence to: {:?}", presence.status);
        
        // Simulate presence update
        tokio::time::sleep(Duration::from_millis(30)).await;
        
        self.current_presence = Some(presence.clone());
        self.emit_event(XmppEvent::PresenceUpdated(presence)).await;
        
        Ok(())
    }
    
    /// Get current presence
    pub fn get_presence(&self) -> Option<&Presence> {
        self.current_presence.as_ref()
    }
    
    /// Join a chat room
    pub async fn join_room(&mut self, room_jid: &str, nickname: &str) -> Result<()> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        info!("Joining chat room: {} as {}", room_jid, nickname);
        
        // Simulate joining room
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let chat_room = ChatRoom {
            jid: room_jid.to_string(),
            name: room_jid.split('@').next().unwrap_or(room_jid).to_string(),
            description: None,
            subject: None,
            participants: vec![nickname.to_string()],
            joined: true,
        };
        
        self.chat_rooms.insert(room_jid.to_string(), chat_room);
        
        info!("Successfully joined room: {}", room_jid);
        
        Ok(())
    }
    
    /// Leave a chat room
    pub async fn leave_room(&mut self, room_jid: &str) -> Result<()> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        info!("Leaving chat room: {}", room_jid);
        
        // Simulate leaving room
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        self.chat_rooms.remove(room_jid);
        
        info!("Successfully left room: {}", room_jid);
        
        Ok(())
    }
    
    /// Get list of joined chat rooms
    pub fn get_chat_rooms(&self) -> Vec<&ChatRoom> {
        self.chat_rooms.values().collect()
    }
    
    /// Add a contact to the roster
    pub async fn add_contact(&mut self, jid: &str, name: Option<String>, groups: Vec<String>) -> Result<()> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        info!("Adding contact: {}", jid);
        
        // Simulate adding contact
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let contact = Contact {
            jid: jid.to_string(),
            name,
            subscription: "none".to_string(),
            groups,
            presence: None,
        };
        
        self.contacts.insert(jid.to_string(), contact.clone());
        self.emit_event(XmppEvent::ContactUpdated(contact)).await;
        
        info!("Successfully added contact: {}", jid);
        
        Ok(())
    }
    
    /// Remove a contact from the roster
    pub async fn remove_contact(&mut self, jid: &str) -> Result<()> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        info!("Removing contact: {}", jid);
        
        // Simulate removing contact
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        self.contacts.remove(jid);
        
        info!("Successfully removed contact: {}", jid);
        
        Ok(())
    }
    
    /// Get list of contacts
    pub fn get_contacts(&self) -> Vec<&Contact> {
        self.contacts.values().collect()
    }
    
    /// Set event callback for receiving events
    pub fn set_event_callback(&mut self, callback: EventCallback) {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.event_tx = Some(tx);
        
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                callback(event);
            }
        });
    }
    
    /// Emit an event
    async fn emit_event(&self, event: XmppEvent) {
        if let Some(tx) = &self.event_tx {
            if let Err(e) = tx.send(event) {
                error!("Failed to send event: {}", e);
            }
        }
    }
    
    /// Get server information
    pub async fn get_server_info(&self) -> Result<HashMap<String, String>> {
        if !self.is_connected {
            return Err(OpenFireError::ConnectionError {
                message: "Not connected to server".to_string(),
            });
        }
        
        // Simulate getting server info
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let mut info = HashMap::new();
        info.insert("server".to_string(), self.config.server.clone());
        info.insert("domain".to_string(), self.config.domain.clone());
        info.insert("port".to_string(), self.config.port.to_string());
        info.insert("version".to_string(), "OpenFire 4.7.0".to_string());
        
        Ok(info)
    }
}

// Utility functions
fn generate_message_id() -> String {
    let timestamp = current_timestamp();
    format!("msg_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Credentials;

    #[tokio::test]
    async fn test_client_creation() {
        let config = Config::default();
        let client = OpenFireClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_message_creation() {
        let msg = Message::new_chat(
            "user1@localhost".to_string(),
            "user2@localhost".to_string(),
            "Hello World!".to_string(),
        );
        
        assert_eq!(msg.message_type, MessageType::Chat);
        assert_eq!(msg.body, "Hello World!");
        assert_eq!(msg.from, "user1@localhost");
        assert_eq!(msg.to, "user2@localhost");
    }

    #[tokio::test]
    async fn test_presence_creation() {
        let presence = Presence::new("user@localhost".to_string(), PresenceStatus::Available)
            .with_message("Working".to_string())
            .with_priority(5);
        
        assert_eq!(presence.status, PresenceStatus::Available);
        assert_eq!(presence.status_message, Some("Working".to_string()));
        assert_eq!(presence.priority, 5);
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let config = Config::default();
        let mut client = OpenFireClient::new(config).unwrap();
        
        let creds = Credentials::new("testuser".to_string(), "testpass".to_string());
        
        // Test connection
        assert!(client.connect(creds).await.is_ok());
        assert!(client.is_connected());
        
        // Test disconnection
        assert!(client.disconnect().await.is_ok());
        assert!(!client.is_connected());
    }
}