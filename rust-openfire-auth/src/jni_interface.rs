//! JNI interface for Java integration with the OpenFire authentication library

use crate::auth::{AuthResult, Credentials};
use crate::communication::{OpenFireClient, Message, Presence, PresenceStatus};
use crate::config::Config;
use crate::error::{OpenFireError, Result};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring, JNI_TRUE, JNI_FALSE};
use jni::JNIEnv;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

// Global runtime for async operations
lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().expect("Failed to create Tokio runtime");
    static ref CLIENTS: Arc<Mutex<HashMap<i64, OpenFireClient>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref CLIENT_COUNTER: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
}

/// Convert Rust string to Java string
fn rust_string_to_jstring(env: &mut JNIEnv, s: &str) -> Result<jstring> {
    env.new_string(s)
        .map(|js| js.into_raw())
        .map_err(|e| OpenFireError::JniError {
            message: format!("Failed to create Java string: {}", e),
        })
}

/// Convert Java string to Rust string
fn jstring_to_rust_string(env: &mut JNIEnv, js: JString) -> Result<String> {
    env.get_string(&js)
        .map(|s| s.into())
        .map_err(|e| OpenFireError::JniError {
            message: format!("Failed to convert Java string: {}", e),
        })
}

/// Convert AuthResult to JSON string for Java
fn auth_result_to_json(result: &AuthResult) -> Result<String> {
    serde_json::to_string(result).map_err(|e| OpenFireError::SerializationError {
        message: format!("Failed to serialize auth result: {}", e),
    })
}

/// Convert Message to JSON string for Java
fn message_to_json(message: &Message) -> Result<String> {
    serde_json::to_string(message).map_err(|e| OpenFireError::SerializationError {
        message: format!("Failed to serialize message: {}", e),
    })
}

/// Convert Presence to JSON string for Java
fn presence_to_json(presence: &Presence) -> Result<String> {
    serde_json::to_string(presence).map_err(|e| OpenFireError::SerializationError {
        message: format!("Failed to serialize presence: {}", e),
    })
}

/// Initialize the OpenFire library
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_initialize(
    mut _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    match crate::init() {
        Ok(_) => {
            info!("OpenFire Auth library initialized successfully");
            JNI_TRUE
        }
        Err(e) => {
            error!("Failed to initialize OpenFire Auth library: {}", e);
            JNI_FALSE
        }
    }
}

/// Create a new OpenFire client instance
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_createClient(
    mut env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> jlong {
    let config_str = match jstring_to_rust_string(&mut env, config_json) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert config string: {}", e);
            return -1;
        }
    };

    let config = match Config::from_json(&config_str) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse config: {}", e);
            return -1;
        }
    };

    let client = match OpenFireClient::new(config) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create client: {}", e);
            return -1;
        }
    };

    let mut counter = CLIENT_COUNTER.lock().unwrap();
    *counter += 1;
    let client_id = *counter;

    let mut clients = CLIENTS.lock().unwrap();
    clients.insert(client_id, client);

    info!("Created OpenFire client with ID: {}", client_id);
    client_id
}

/// Destroy an OpenFire client instance
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_destroyClient(
    _env: JNIEnv,
    _class: JClass,
    client_id: jlong,
) -> jboolean {
    let mut clients = CLIENTS.lock().unwrap();
    
    if let Some(mut client) = clients.remove(&client_id) {
        // Disconnect if still connected
        if client.is_connected() {
            let _ = RUNTIME.block_on(client.disconnect());
        }
        info!("Destroyed OpenFire client with ID: {}", client_id);
        JNI_TRUE
    } else {
        warn!("Attempted to destroy non-existent client: {}", client_id);
        JNI_FALSE
    }
}

/// Connect to OpenFire server
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_connect(
    mut env: JNIEnv,
    _class: JClass,
    client_id: jlong,
    username: JString,
    password: JString,
    domain: JString,
) -> jstring {
    let username_str = match jstring_to_rust_string(&mut env, username) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert username: {}", e);
            return std::ptr::null_mut();
        }
    };

    let password_str = match jstring_to_rust_string(&mut env, password) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert password: {}", e);
            return std::ptr::null_mut();
        }
    };

    let domain_str = match jstring_to_rust_string(&mut env, domain) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert domain: {}", e);
            return std::ptr::null_mut();
        }
    };

    let mut clients = CLIENTS.lock().unwrap();
    let client = match clients.get_mut(&client_id) {
        Some(c) => c,
        None => {
            error!("Client not found: {}", client_id);
            return std::ptr::null_mut();
        }
    };

    let credentials = if domain_str.is_empty() {
        Credentials::new(username_str, password_str)
    } else {
        Credentials::with_domain(username_str, password_str, domain_str.clone())
    };

    let result = RUNTIME.block_on(client.connect(credentials.clone()));
    
    match result {
        Ok(_) => {
            let success_result = AuthResult::success(
                format!("{}@{}", credentials.username, domain_str),
                Some("session_id".to_string()),
                100,
            );
            
            match auth_result_to_json(&success_result) {
                Ok(json) => {
                    match rust_string_to_jstring(&mut env, &json) {
                        Ok(jstr) => jstr,
                        Err(_) => std::ptr::null_mut(),
                    }
                }
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(e) => {
            let error_result = AuthResult::failure(e.to_string(), 100);
            
            match auth_result_to_json(&error_result) {
                Ok(json) => {
                    match rust_string_to_jstring(&mut env, &json) {
                        Ok(jstr) => jstr,
                        Err(_) => std::ptr::null_mut(),
                    }
                }
                Err(_) => std::ptr::null_mut(),
            }
        }
    }
}

/// Disconnect from OpenFire server
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_disconnect(
    _env: JNIEnv,
    _class: JClass,
    client_id: jlong,
) -> jboolean {
    let mut clients = CLIENTS.lock().unwrap();
    let client = match clients.get_mut(&client_id) {
        Some(c) => c,
        None => {
            error!("Client not found: {}", client_id);
            return JNI_FALSE;
        }
    };

    match RUNTIME.block_on(client.disconnect()) {
        Ok(_) => {
            info!("Client {} disconnected successfully", client_id);
            JNI_TRUE
        }
        Err(e) => {
            error!("Failed to disconnect client {}: {}", client_id, e);
            JNI_FALSE
        }
    }
}

/// Check if client is connected
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_isConnected(
    _env: JNIEnv,
    _class: JClass,
    client_id: jlong,
) -> jboolean {
    let clients = CLIENTS.lock().unwrap();
    let client = match clients.get(&client_id) {
        Some(c) => c,
        None => {
            error!("Client not found: {}", client_id);
            return JNI_FALSE;
        }
    };

    if client.is_connected() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// Send a chat message
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_sendMessage(
    mut env: JNIEnv,
    _class: JClass,
    client_id: jlong,
    to: JString,
    body: JString,
) -> jstring {
    let to_str = match jstring_to_rust_string(&mut env, to) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert 'to' field: {}", e);
            return std::ptr::null_mut();
        }
    };

    let body_str = match jstring_to_rust_string(&mut env, body) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert message body: {}", e);
            return std::ptr::null_mut();
        }
    };

    let clients = CLIENTS.lock().unwrap();
    let client = match clients.get(&client_id) {
        Some(c) => c,
        None => {
            error!("Client not found: {}", client_id);
            return std::ptr::null_mut();
        }
    };

    match RUNTIME.block_on(client.send_message(&to_str, &body_str)) {
        Ok(message_id) => {
            match rust_string_to_jstring(&mut env, &message_id) {
                Ok(jstr) => jstr,
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(e) => {
            error!("Failed to send message: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Set presence status
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_setPresence(
    mut env: JNIEnv,
    _class: JClass,
    client_id: jlong,
    status: jint,
    message: JString,
) -> jboolean {
    let status_enum = match status {
        0 => PresenceStatus::Available,
        1 => PresenceStatus::Away,
        2 => PresenceStatus::ExtendedAway,
        3 => PresenceStatus::DoNotDisturb,
        4 => PresenceStatus::Unavailable,
        5 => PresenceStatus::Invisible,
        _ => {
            error!("Invalid presence status: {}", status);
            return JNI_FALSE;
        }
    };

    let message_str = if message.is_null() {
        None
    } else {
        match jstring_to_rust_string(&mut env, message) {
            Ok(s) if s.is_empty() => None,
            Ok(s) => Some(s),
            Err(e) => {
                error!("Failed to convert presence message: {}", e);
                return JNI_FALSE;
            }
        }
    };

    let mut clients = CLIENTS.lock().unwrap();
    let client = match clients.get_mut(&client_id) {
        Some(c) => c,
        None => {
            error!("Client not found: {}", client_id);
            return JNI_FALSE;
        }
    };

    match RUNTIME.block_on(client.set_presence(status_enum, message_str)) {
        Ok(_) => {
            info!("Presence updated for client {}", client_id);
            JNI_TRUE
        }
        Err(e) => {
            error!("Failed to set presence for client {}: {}", client_id, e);
            JNI_FALSE
        }
    }
}

/// Get current presence as JSON
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_getPresence(
    mut env: JNIEnv,
    _class: JClass,
    client_id: jlong,
) -> jstring {
    let clients = CLIENTS.lock().unwrap();
    let client = match clients.get(&client_id) {
        Some(c) => c,
        None => {
            error!("Client not found: {}", client_id);
            return std::ptr::null_mut();
        }
    };

    match client.get_presence() {
        Some(presence) => {
            match presence_to_json(presence) {
                Ok(json) => {
                    match rust_string_to_jstring(&mut env, &json) {
                        Ok(jstr) => jstr,
                        Err(_) => std::ptr::null_mut(),
                    }
                }
                Err(_) => std::ptr::null_mut(),
            }
        }
        None => std::ptr::null_mut(),
    }
}

/// Join a chat room
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_joinRoom(
    mut env: JNIEnv,
    _class: JClass,
    client_id: jlong,
    room_jid: JString,
    nickname: JString,
) -> jboolean {
    let room_jid_str = match jstring_to_rust_string(&mut env, room_jid) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert room JID: {}", e);
            return JNI_FALSE;
        }
    };

    let nickname_str = match jstring_to_rust_string(&mut env, nickname) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert nickname: {}", e);
            return JNI_FALSE;
        }
    };

    let mut clients = CLIENTS.lock().unwrap();
    let client = match clients.get_mut(&client_id) {
        Some(c) => c,
        None => {
            error!("Client not found: {}", client_id);
            return JNI_FALSE;
        }
    };

    match RUNTIME.block_on(client.join_room(&room_jid_str, &nickname_str)) {
        Ok(_) => {
            info!("Joined room {} as {}", room_jid_str, nickname_str);
            JNI_TRUE
        }
        Err(e) => {
            error!("Failed to join room: {}", e);
            JNI_FALSE
        }
    }
}

/// Get library version
#[no_mangle]
pub extern "system" fn Java_org_jivesoftware_spark_openfire_OpenFireAuthNative_getVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    match rust_string_to_jstring(&mut env, crate::VERSION) {
        Ok(jstr) => jstr,
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::new_chat(
            "user1@localhost".to_string(),
            "user2@localhost".to_string(),
            "Hello".to_string(),
        );
        
        let json = message_to_json(&msg).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("user1@localhost"));
    }

    #[test]
    fn test_presence_serialization() {
        let presence = Presence::new("user@localhost".to_string(), PresenceStatus::Available);
        let json = presence_to_json(&presence).unwrap();
        assert!(json.contains("Available"));
        assert!(json.contains("user@localhost"));
    }
}