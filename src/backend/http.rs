use serde::Deserialize;
use thiserror::Error;

use super::core::{
    AuthResponse, BackendError, ConnectionInfo, PeerStatus, Server, UserInfo, VpnBackend,
};

// ---------------------------------------------------------------------------
// Erreur HTTP
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
}

impl From<ApiError> for BackendError {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::Request(e) => BackendError::Request(e.to_string()),
            ApiError::Api(msg) => BackendError::Api(msg),
        }
    }
}

// ---------------------------------------------------------------------------
// Formats de réponse internes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ApiSuccess<T> {
    data: T,
}

#[derive(Deserialize)]
struct ApiErrorResp {
    error: String,
}

#[derive(Deserialize)]
struct AuthData {
    token: String,
    user: UserInfo,
}

#[derive(Deserialize)]
struct ProfileUpdateResp {
    user: UserInfo,
}

// ---------------------------------------------------------------------------
// Struct
// ---------------------------------------------------------------------------

pub struct HttpBackend {
    pub(crate) base_url: String,
    pub(crate) token: String,
    pub(crate) client: reqwest::blocking::Client,
}

impl HttpBackend {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = token.to_string();
    }

    fn parse_error(&self, resp: reqwest::blocking::Response) -> ApiError {
        match resp.json::<ApiErrorResp>() {
            Ok(e) => ApiError::Api(e.error),
            Err(_) => ApiError::Api("unknown error".into()),
        }
    }
}

impl VpnBackend for HttpBackend {
    type Error = ApiError;

    fn login(&self, username: &str, password: &str) -> Result<AuthResponse, Self::Error> {
        let resp = self
            .client
            .post(format!("{}/auth/login", self.base_url))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        let body: ApiSuccess<AuthData> = resp.json()?;
        Ok(AuthResponse {
            token: body.data.token,
            user: body.data.user,
        })
    }

    fn register(&self, username: &str, password: &str) -> Result<AuthResponse, Self::Error> {
        let resp = self
            .client
            .post(format!("{}/auth/register", self.base_url))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        let body: ApiSuccess<AuthData> = resp.json()?;
        Ok(AuthResponse {
            token: body.data.token,
            user: body.data.user,
        })
    }

    fn logout(&self, token: &str) -> Result<(), Self::Error> {
        let resp = self
            .client
            .post(format!("{}/auth/logout", self.base_url))
            .bearer_auth(token)
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        Ok(())
    }

    fn list_servers(&self) -> Result<Vec<Server>, Self::Error> {
        let resp = self
            .client
            .get(format!("{}/vpn/servers", self.base_url))
            .bearer_auth(&self.token)
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        let body: ApiSuccess<Vec<Server>> = resp.json()?;
        Ok(body.data)
    }

    fn connect(&self, server_id: u64) -> Result<ConnectionInfo, Self::Error> {
        let resp = self
            .client
            .post(format!("{}/vpn/connect", self.base_url))
            .bearer_auth(&self.token)
            .json(&serde_json::json!({ "server_id": server_id }))
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        let body: ApiSuccess<ConnectionInfo> = resp.json()?;
        Ok(body.data)
    }

    fn disconnect(&self, server_id: u64) -> Result<(), Self::Error> {
        let resp = self
            .client
            .post(format!("{}/vpn/disconnect", self.base_url))
            .bearer_auth(&self.token)
            .json(&serde_json::json!({ "server_id": server_id }))
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        Ok(())
    }

    fn peer_status(&self) -> Result<Vec<PeerStatus>, Self::Error> {
        let resp = self
            .client
            .get(format!("{}/vpn/status", self.base_url))
            .bearer_auth(&self.token)
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        let body: ApiSuccess<Vec<PeerStatus>> = resp.json()?;
        Ok(body.data)
    }

    fn update_profile(&self, username: &str, password: &str) -> Result<UserInfo, Self::Error> {
        let resp = self
            .client
            .put(format!("{}/profile/update", self.base_url))
            .bearer_auth(&self.token)
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        let body: ProfileUpdateResp = resp.json()?;
        Ok(body.user)
    }

    fn delete_account(&self) -> Result<(), Self::Error> {
        let resp = self
            .client
            .delete(format!("{}/profile/delete", self.base_url))
            .bearer_auth(&self.token)
            .send()?;

        if !resp.status().is_success() {
            return Err(self.parse_error(resp));
        }

        Ok(())
    }
}
