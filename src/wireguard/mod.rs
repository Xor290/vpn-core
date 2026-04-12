pub mod config;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WireGuardError {
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("invalid config format")]
    InvalidFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub private_key: String,
    pub address: String,
    pub dns: String,
    pub peer_public_key: String,
    pub endpoint: String,
    pub allowed_ips: String,
    pub persistent_keepalive: u16,
}
