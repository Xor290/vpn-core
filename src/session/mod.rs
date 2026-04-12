pub mod manager;
use zeroize::{Zeroize, ZeroizeOnDrop};

use thiserror::Error;

use crate::backend::core::{Server, UserInfo, VpnBackend};
use crate::wireguard::{WireGuardConfig, WireGuardError};

// ---------------------------------------------------------------------------
// Struct générique
// ---------------------------------------------------------------------------
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Session<B: VpnBackend> {
    #[zeroize(skip)]
    pub(crate) backend: B,
    pub(crate) token: String,
    pub(crate) user: UserInfo,
    pub(crate) current_server: Option<Server>,
    pub(crate) config: Option<WireGuardConfig>,
}

// ---------------------------------------------------------------------------
// Erreur de session
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SessionError<E: std::error::Error + 'static> {
    #[error("backend error: {0}")]
    Backend(E),
    #[error("wireguard error: {0}")]
    WireGuard(#[from] WireGuardError),
    #[error("not connected")]
    NotConnected,
}
