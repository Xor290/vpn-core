use std::future::Future;

use crate::backend::core::{AuthResponse, ConnectionInfo, PeerStatus, Server, UserInfo};

pub trait AsyncVpnBackend {
    type Error: std::error::Error + Send + Sync + 'static;

    fn login(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<AuthResponse, Self::Error>> + Send;

    fn register(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<AuthResponse, Self::Error>> + Send;

    fn logout(&self, token: &str) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn list_servers(&self) -> impl Future<Output = Result<Vec<Server>, Self::Error>> + Send;

    fn connect(&self, server_id: u64) -> impl Future<Output = Result<ConnectionInfo, Self::Error>> + Send;

    fn disconnect(&self, server_id: u64) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn peer_status(&self) -> impl Future<Output = Result<Vec<PeerStatus>, Self::Error>> + Send;

    fn update_profile(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<UserInfo, Self::Error>> + Send;

    fn delete_account(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
