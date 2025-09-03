//! Configuration management for OpenFire connections

use serde::{Deserialize, Serialize};
use crate::error::{OpenFireError, Result};
use std::path::Path;

/// OpenFire connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server hostname or IP address
    pub server: String,
    
    /// Server port (typically 5222 for XMPP)
    pub port: u16,
    
    /// Domain/realm for authentication
    pub domain: String,
    
    /// Whether to use TLS/SSL
    pub use_tls: bool,
    
    /// Whether to verify TLS certificates
    pub verify_certificates: bool,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Authentication timeout in seconds
    pub auth_timeout: u64,
    
    /// Resource identifier for this client
    pub resource: String,
    
    /// Priority for presence (0-127)
    pub priority: i8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: "localhost".to_string(),
            port: 5222,
            domain: "localhost".to_string(),
            use_tls: true,
            verify_certificates: true,
            connection_timeout: 30,
            auth_timeout: 10,
            resource: "SparkRust".to_string(),
            priority: 1,
        }
    }
}

impl Config {
    /// Create a new configuration
    pub fn new(server: String, domain: String) -> Self {
        Self {
            server,
            domain,
            ..Default::default()
        }
    }
    
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| OpenFireError::ConfigError {
                message: format!("Failed to read config file: {}", e),
            })?;
        
        toml::from_str(&content)
            .map_err(|e| OpenFireError::ConfigError {
                message: format!("Failed to parse config file: {}", e),
            })
    }
    
    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| OpenFireError::ConfigError {
                message: format!("Failed to serialize config: {}", e),
            })?;
        
        std::fs::write(path, content)
            .map_err(|e| OpenFireError::ConfigError {
                message: format!("Failed to write config file: {}", e),
            })
    }
    
    /// Create configuration from JSON (for Java interop)
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| OpenFireError::ConfigError {
                message: format!("Failed to parse JSON config: {}", e),
            })
    }
    
    /// Convert configuration to JSON (for Java interop)
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| OpenFireError::ConfigError {
                message: format!("Failed to serialize config to JSON: {}", e),
            })
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.server.is_empty() {
            return Err(OpenFireError::ConfigError {
                message: "Server cannot be empty".to_string(),
            });
        }
        
        if self.domain.is_empty() {
            return Err(OpenFireError::ConfigError {
                message: "Domain cannot be empty".to_string(),
            });
        }
        
        if self.port == 0 {
            return Err(OpenFireError::ConfigError {
                message: "Port must be greater than 0".to_string(),
            });
        }
        
        if self.connection_timeout == 0 || self.auth_timeout == 0 {
            return Err(OpenFireError::ConfigError {
                message: "Timeout values must be greater than 0".to_string(),
            });
        }
        
        if self.priority < -128 || self.priority > 127 {
            return Err(OpenFireError::ConfigError {
                message: "Priority must be between -128 and 127".to_string(),
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server, "localhost");
        assert_eq!(config.port, 5222);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Test empty server
        config.server = "".to_string();
        assert!(config.validate().is_err());
        
        // Test empty domain
        config.server = "localhost".to_string();
        config.domain = "".to_string();
        assert!(config.validate().is_err());
        
        // Test zero port
        config.domain = "localhost".to_string();
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_json_serialization() {
        let config = Config::default();
        let json = config.to_json().unwrap();
        let parsed = Config::from_json(&json).unwrap();
        
        assert_eq!(config.server, parsed.server);
        assert_eq!(config.domain, parsed.domain);
        assert_eq!(config.port, parsed.port);
    }
}