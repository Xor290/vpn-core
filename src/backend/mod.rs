pub mod async_core;
pub mod core;
pub mod grpc;

#[cfg(feature = "http-async")]
pub mod http_async;
pub use async_core::{
    AuthResponseAsync, BackendErrorAsync, ConnectionInfoAsync, PeerStatusAsync, ServerAsync, UserInfoAsync,
};
#[cfg(feature = "http-backend")]
pub mod http;
pub use core::{
    AuthResponse, BackendError, ConnectionInfo, PeerStatus, Server, UserInfo, VpnBackend,
};

#[cfg(feature = "http-backend")]
pub use http::HttpBackend;
