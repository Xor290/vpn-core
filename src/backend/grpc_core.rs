use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

pub struct UserInfoGrpc {
    pub username: String,
    pub password: String,
}

pub struct AuthResponseGrpc {
    pub token: String,
    pub username: String,
}
