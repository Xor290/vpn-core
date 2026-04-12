pub mod backend;
pub mod session;
pub mod wireguard;

/// Alias de commodité pour une session utilisant le backend HTTP par défaut.
#[cfg(feature = "http-backend")]
pub type HttpSession = session::Session<backend::HttpBackend>;
