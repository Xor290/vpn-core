pub mod async_core;
pub mod core;

#[cfg(feature = "http-async")]
pub mod http_async;

#[cfg(feature = "http-backend")]
pub mod http;

#[cfg(feature = "grpc-backend")]
pub mod grpc;

pub use async_core::{
    AsyncVpnBackend, AuthResponseAsync, BackendErrorAsync, ConnectionInfoAsync, PeerStatusAsync,
    ServerAsync, UserInfoAsync,
};
pub use core::{
    AuthResponse, BackendError, ConnectionInfo, PeerStatus, Server, UserInfo, VpnBackend,
};

#[cfg(feature = "http-backend")]
pub use http::HttpBackend;

#[cfg(feature = "http-async")]
pub use http_async::HttpAsyncBackend;
