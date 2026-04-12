pub mod backend;

#[cfg(feature = "http-backend")]
pub mod http;

pub use backend::{AuthResponse, BackendError, ConnectionInfo, PeerStatus, Server, UserInfo, VpnBackend};

#[cfg(feature = "http-backend")]
pub use http::HttpBackend;
