pub mod backend;
pub mod session;
pub mod wireguard;

#[cfg(feature = "http-backend")]
pub type HttpSession = session::Session<backend::HttpBackend>;
