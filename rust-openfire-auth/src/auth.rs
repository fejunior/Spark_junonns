//! Authentication management for OpenFire connections

use crate::config::Config;
use crate::error::{OpenFireError, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// User credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub domain: Option<String>,
}

impl Credentials {
    /// Create new credentials
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            domain: None,
        }
    }
    
    /// Create credentials with domain
    pub fn with_domain(username: String, password: String, domain: String) -> Self {
        Self {
            username,
            password,
            domain: Some(domain),
        }
    }
    
    /// Get the full JID (username@domain)
    pub fn get_jid(&self, default_domain: &str) -> String {
        let binding = default_domain.to_string();
        let domain = self.domain.as_ref().unwrap_or(&binding);
        format!("{}@{}", self.username, domain)
    }
    
    /// Validate credentials format
    pub fn validate(&self) -> Result<()> {
        if self.username.is_empty() {
            return Err(OpenFireError::InvalidCredentials {
                message: "Username cannot be empty".to_string(),
            });
        }
        
        if self.password.is_empty() {
            return Err(OpenFireError::InvalidCredentials {
                message: "Password cannot be empty".to_string(),
            });
        }
        
        // Basic username validation (no spaces, basic characters)
        if self.username.contains(' ') || self.username.contains('@') {
            return Err(OpenFireError::InvalidCredentials {
                message: "Username contains invalid characters".to_string(),
            });
        }
        
        Ok(())
    }
}

/// Authentication state
#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Disconnected,
    Connecting,
    Authenticating,
    Authenticated,
    Failed(String),
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub success: bool,
    pub message: String,
    pub full_jid: Option<String>,
    pub session_id: Option<String>,
    pub auth_time_ms: u64,
}

impl AuthResult {
    pub fn success(full_jid: String, session_id: Option<String>, auth_time_ms: u64) -> Self {
        Self {
            success: true,
            message: "Authentication successful".to_string(),
            full_jid: Some(full_jid),
            session_id,
            auth_time_ms,
        }
    }
    
    pub fn failure(message: String, auth_time_ms: u64) -> Self {
        Self {
            success: false,
            message,
            full_jid: None,
            session_id: None,
            auth_time_ms,
        }
    }
}

/// Authentication manager
pub struct AuthManager {
    config: Config,
    state: AuthState,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        
        Ok(Self {
            config,
            state: AuthState::Disconnected,
        })
    }
    
    /// Get current authentication state
    pub fn get_state(&self) -> &AuthState {
        &self.state
    }
    
    /// Check if currently authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, AuthState::Authenticated)
    }
    
    /// Authenticate with OpenFire server
    pub async fn authenticate(&mut self, credentials: Credentials) -> Result<AuthResult> {
        let start_time = Instant::now();
        
        info!("Starting authentication for user: {}", credentials.username);
        
        // Validate credentials
        credentials.validate()?;
        
        self.state = AuthState::Connecting;
        
        // Simulate connection process (in real implementation, this would use XMPP)
        match self.perform_authentication(&credentials).await {
            Ok(result) => {
                self.state = AuthState::Authenticated;
                info!("Authentication successful for user: {}", credentials.username);
                Ok(result)
            }
            Err(e) => {
                let error_msg = format!("Authentication failed: {}", e);
                self.state = AuthState::Failed(error_msg.clone());
                error!("Authentication failed for user {}: {}", credentials.username, e);
                
                Ok(AuthResult::failure(
                    error_msg,
                    start_time.elapsed().as_millis() as u64,
                ))
            }
        }
    }
    
    /// Perform the actual authentication (placeholder implementation)
    async fn perform_authentication(&mut self, credentials: &Credentials) -> Result<AuthResult> {
        let start_time = Instant::now();
        
        self.state = AuthState::Authenticating;
        
        // Simulate authentication delay
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // In a real implementation, this would:
        // 1. Connect to the XMPP server
        // 2. Negotiate TLS if required
        // 3. Send authentication request (SASL)
        // 4. Handle authentication response
        // 5. Bind resource and establish session
        
        // For now, simulate successful authentication
        let full_jid = credentials.get_jid(&self.config.domain);
        let session_id = Some(format!("session_{}", uuid::Uuid::new_v4()));
        let auth_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Simulate some basic validation
        if credentials.username == "invalid" {
            return Err(OpenFireError::AuthenticationFailed {
                message: "Invalid username".to_string(),
            });
        }
        
        if credentials.password == "wrong" {
            return Err(OpenFireError::AuthenticationFailed {
                message: "Invalid password".to_string(),
            });
        }
        
        // Check connection timeout
        if auth_time_ms > (self.config.auth_timeout * 1000) {
            return Err(OpenFireError::TimeoutError {
                seconds: self.config.auth_timeout,
            });
        }
        
        info!("Authentication completed in {}ms", auth_time_ms);
        
        Ok(AuthResult::success(full_jid, session_id, auth_time_ms))
    }
    
    /// Disconnect and clear authentication state
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from OpenFire server");
        
        // In a real implementation, this would properly close the XMPP connection
        self.state = AuthState::Disconnected;
        
        Ok(())
    }
    
    /// Test connection to the server without authenticating
    pub async fn test_connection(&self) -> Result<bool> {
        info!("Testing connection to {}:{}", self.config.server, self.config.port);
        
        // Simulate connection test
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // In a real implementation, this would attempt to connect to the server
        // and check if it's reachable and responding to XMPP requests
        
        Ok(true)
    }
    
    /// Get server information
    pub fn get_server_info(&self) -> String {
        format!("{}:{} (domain: {})", self.config.server, self.config.port, self.config.domain)
    }
}

// Add UUID dependency for session IDs
mod uuid {
    use std::fmt;
    
    pub struct Uuid(String);
    
    impl Uuid {
        pub fn new_v4() -> Self {
            // Simple UUID generation for demo purposes
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            
            Self(format!("uuid-{:x}", timestamp))
        }
    }
    
    impl fmt::Display for Uuid {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_manager_creation() {
        let config = Config::default();
        let auth_manager = AuthManager::new(config);
        assert!(auth_manager.is_ok());
    }

    #[tokio::test]
    async fn test_credentials_validation() {
        let valid_creds = Credentials::new("testuser".to_string(), "testpass".to_string());
        assert!(valid_creds.validate().is_ok());
        
        let invalid_creds = Credentials::new("".to_string(), "testpass".to_string());
        assert!(invalid_creds.validate().is_err());
        
        let invalid_creds2 = Credentials::new("test user".to_string(), "testpass".to_string());
        assert!(invalid_creds2.validate().is_err());
    }

    #[tokio::test]
    async fn test_authentication_success() {
        let config = Config::default();
        let mut auth_manager = AuthManager::new(config).unwrap();
        
        let creds = Credentials::new("testuser".to_string(), "testpass".to_string());
        let result = auth_manager.authenticate(creds).await.unwrap();
        
        assert!(result.success);
        assert!(auth_manager.is_authenticated());
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let config = Config::default();
        let mut auth_manager = AuthManager::new(config).unwrap();
        
        let creds = Credentials::new("invalid".to_string(), "testpass".to_string());
        let result = auth_manager.authenticate(creds).await.unwrap();
        
        assert!(!result.success);
        assert!(!auth_manager.is_authenticated());
    }
}