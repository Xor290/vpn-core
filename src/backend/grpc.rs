use crate::backend::{AuthResponse, ConnectionInfo, PeerStatus, Server, UserInfo};

#[tarpc::service]
pub trait VpnService {
    async fn login(username: String, password: String) -> Result<AuthResponse, String>;
    async fn register(username: String, password: String) -> Result<AuthResponse, String>;
    async fn logout(token: String) -> Result<(), String>;
    async fn list_servers() -> Result<Vec<Server>, String>;
    async fn connect(server_id: u64) -> Result<ConnectionInfo, String>;
    async fn disconnect(server_id: u64) -> Result<(), String>;
    async fn peer_status() -> Result<Vec<PeerStatus>, String>;
    async fn update_profile(username: String, password: String) -> Result<UserInfo, String>;
    async fn delete_account() -> Result<(), String>;
}
