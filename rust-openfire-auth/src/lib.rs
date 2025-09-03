//! OpenFire Authentication and Communication Library
//! 
//! This library provides authentication and communication functionality
//! for connecting to OpenFire XMPP servers from Rust.

pub mod auth;
pub mod communication;
pub mod config;
pub mod error;
pub mod jni_interface;

pub use auth::AuthManager;
pub use communication::OpenFireClient;
pub use config::Config;
pub use error::{OpenFireError, Result};

use log::info;

/// Initialize the OpenFire authentication library
pub fn init() -> Result<()> {
    env_logger::init();
    info!("OpenFire Auth library initialized");
    Ok(())
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}