use std::future::Future;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct UserInfoAsync {
    pub id: u64,
    pub username: String,
}

#[derive(Clone)]
pub struct AuthResponseAsync {
    pub token: String,
    pub user: UserInfoAsync,
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct ServerAsync {
    pub id: u64,
    pub name: String,
    pub country: String,
    pub ip: String,
    pub public_key: String,
    pub listen_port: u16,
    pub subnet: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfoAsync {
    pub peer_ip: String,
    pub config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatusAsync {
    pub id: u64,
    pub user_id: u64,
    pub server_id: u64,
    pub public_key: String,
    pub allowed_ip: String,
    pub server: ServerAsync,
}

#[derive(Debug, Error)]
pub enum BackendErrorAsync {
    #[error("request failed: {0}")]
    Request(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("unexpected response: {0}")]
    Parse(String),
}

pub trait AsyncVpnBackend {
    type Error: std::error::Error + Send + Sync + 'static;

    fn login(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<AuthResponseAsync, Self::Error>> + Send;

    fn register(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<AuthResponseAsync, Self::Error>> + Send;

    fn logout(&self, token: &str) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Appelé après login/register pour stocker le token dans le backend.
    /// Implémentation par défaut : no-op.
    fn set_auth_token(&mut self, _token: &str) {}

    fn list_servers(&self) -> impl Future<Output = Result<Vec<ServerAsync>, Self::Error>> + Send;

    fn connect(
        &self,
        server_id: u64,
    ) -> impl Future<Output = Result<ConnectionInfoAsync, Self::Error>> + Send;

    fn disconnect(&self, server_id: u64) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn peer_status(&self)
        -> impl Future<Output = Result<Vec<PeerStatusAsync>, Self::Error>> + Send;

    fn update_profile(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<UserInfoAsync, Self::Error>> + Send;

    fn delete_account(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
