pub mod handlers;
pub use handlers::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
}

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

#[derive(Deserialize)]
struct ApiSuccess {
    data: AuthData,
}

#[derive(Deserialize)]
struct AuthData {
    token: String,
    user: UserInfo,
}

#[derive(Deserialize)]
struct ApiError {
    error: String,
}
