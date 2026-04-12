use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Types communs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: u64,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: u64,
    pub name: String,
    pub country: String,
    pub ip: String,
    pub public_key: String,
    pub listen_port: u16,
    pub subnet: String,
    pub is_active: bool,
}

/// Retourné par `connect()` : contient la config WireGuard brute au format INI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub peer_ip: String,
    /// Config WireGuard au format INI, prête à être parsée par `WireGuardConfig::parse`.
    pub config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatus {
    pub id: u64,
    pub user_id: u64,
    pub server_id: u64,
    pub public_key: String,
    pub allowed_ip: String,
    pub server: Server,
}

// ---------------------------------------------------------------------------
// Erreur générique exposée aux implémenteurs
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("request failed: {0}")]
    Request(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("unexpected response: {0}")]
    Parse(String),
}

// ---------------------------------------------------------------------------
// Trait principal
// ---------------------------------------------------------------------------

/// Abstraction d'un backend VPN.
///
/// Implémenter ce trait suffit pour utiliser `Session<B>` avec n'importe
/// quel backend (REST, gRPC, mock, ...).
///
/// # Exemple minimal
///
/// ```rust,ignore
/// struct MyBackend { token: String }
///
/// impl VpnBackend for MyBackend {
///     type Error = BackendError;
///
///     fn login(&self, username: &str, password: &str) -> Result<AuthResponse, Self::Error> {
///         // appel HTTP, gRPC, etc.
///     }
///     // ...
/// }
/// ```
pub trait VpnBackend {
    type Error: std::error::Error + Send + Sync + 'static;

    // -- Authentification ---------------------------------------------------

    fn login(&self, username: &str, password: &str) -> Result<AuthResponse, Self::Error>;
    fn register(&self, username: &str, password: &str) -> Result<AuthResponse, Self::Error>;
    fn logout(&self, token: &str) -> Result<(), Self::Error>;

    // -- Serveurs & connexion -----------------------------------------------

    fn list_servers(&self) -> Result<Vec<Server>, Self::Error>;

    /// Demande une connexion au serveur `server_id`.
    /// Retourne une `ConnectionInfo` contenant la config WireGuard INI.
    fn connect(&self, server_id: u64) -> Result<ConnectionInfo, Self::Error>;

    fn disconnect(&self, server_id: u64) -> Result<(), Self::Error>;

    // -- Statut -------------------------------------------------------------

    fn peer_status(&self) -> Result<Vec<PeerStatus>, Self::Error>;

    // -- Compte -------------------------------------------------------------

    fn update_profile(&self, username: &str, password: &str) -> Result<UserInfo, Self::Error>;
    fn delete_account(&self) -> Result<(), Self::Error>;
}
