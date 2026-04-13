use super::async_core::{
    AsyncVpnBackend, AuthResponseAsync, BackendErrorAsync, ConnectionInfoAsync, PeerStatusAsync,
    ServerAsync, UserInfoAsync,
};
use serde::Deserialize;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

// ---------------------------------------------------------------------------
// Erreur propre au backend HTTP async
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum HttpAsyncError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
}

impl From<HttpAsyncError> for BackendErrorAsync {
    fn from(e: HttpAsyncError) -> Self {
        match e {
            HttpAsyncError::Request(e) => BackendErrorAsync::Request(e.to_string()),
            HttpAsyncError::Api(msg) => BackendErrorAsync::Api(msg),
        }
    }
}

// ---------------------------------------------------------------------------
// Formats de réponse internes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ApiSuccessAsync<T> {
    data: T,
}

#[derive(Deserialize)]
struct ApiErrorRespAsync {
    error: String,
}

#[derive(Deserialize, Zeroize, ZeroizeOnDrop)]
struct AuthDataAsync {
    token: String,
    user: UserInfoAsync,
}

#[derive(Deserialize)]
struct ProfileUpdateRespAsync {
    user: UserInfoAsync,
}

// ---------------------------------------------------------------------------
// Backend
// ---------------------------------------------------------------------------

pub struct HttpAsyncBackend {
    pub(crate) base_url: String,
    pub(crate) token: Zeroizing<String>,
    pub(crate) client: reqwest::Client,
}

impl HttpAsyncBackend {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: Zeroizing::new(token.to_string()),
            client: reqwest::Client::builder()
                .min_tls_version(reqwest::tls::Version::TLS_1_2)
                .https_only(true)
                .build()
                .expect("failed to build async HTTP client"),
        }
    }

    #[cfg(any(feature = "testing", feature = "testing-async"))]
    pub fn new_insecure(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: Zeroizing::new(token.to_string()),
            client: reqwest::Client::new(),
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = Zeroizing::new(token.to_string());
    }

    fn parse_error(resp_text: &str) -> HttpAsyncError {
        #[derive(Deserialize)]
        struct E {
            error: String,
        }
        match serde_json::from_str::<E>(resp_text) {
            Ok(e) => HttpAsyncError::Api(e.error),
            Err(_) => HttpAsyncError::Api("unknown error".into()),
        }
    }
}

impl AsyncVpnBackend for HttpAsyncBackend {
    type Error = HttpAsyncError;

    fn set_auth_token(&mut self, token: &str) {
        self.set_token(token);
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponseAsync, Self::Error> {
        let resp = self
            .client
            .post(format!("{}/auth/login", self.base_url))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }

        let body: ApiSuccessAsync<AuthDataAsync> = resp.json().await?;
        Ok(AuthResponseAsync {
            token: body.data.token.clone(),
            user: body.data.user.clone(),
        })
    }

    async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponseAsync, Self::Error> {
        let resp = self
            .client
            .post(format!("{}/auth/register", self.base_url))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        let body: ApiSuccessAsync<AuthDataAsync> = resp.json().await?;
        Ok(AuthResponseAsync {
            token: body.data.token.clone(),
            user: body.data.user.clone(),
        })
    }

    async fn logout(&self, token: &str) -> Result<(), Self::Error> {
        let resp = self
            .client
            .post(format!("{}/auth/logout", self.base_url))
            .bearer_auth(token)
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        Ok(())
    }

    async fn list_servers(&self) -> Result<Vec<ServerAsync>, Self::Error> {
        let resp = self
            .client
            .get(format!("{}/vpn/servers", self.base_url))
            .bearer_auth(self.token.as_str())
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        let body: ApiSuccessAsync<Vec<ServerAsync>> = resp.json().await?;
        Ok(body.data)
    }

    async fn connect(&self, server_id: u64) -> Result<ConnectionInfoAsync, Self::Error> {
        let resp = self
            .client
            .post(format!("{}/vpn/connect", self.base_url))
            .bearer_auth(self.token.as_str())
            .json(&serde_json::json!({ "server_id": server_id }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        let body: ApiSuccessAsync<ConnectionInfoAsync> = resp.json().await?;
        Ok(body.data)
    }

    async fn disconnect(&self, server_id: u64) -> Result<(), Self::Error> {
        let resp = self
            .client
            .post(format!("{}/vpn/disconnect", self.base_url))
            .bearer_auth(self.token.as_str())
            .json(&serde_json::json!({ "server_id": server_id }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        Ok(())
    }

    async fn peer_status(&self) -> Result<Vec<PeerStatusAsync>, Self::Error> {
        let resp = self
            .client
            .get(format!("{}/vpn/status", self.base_url))
            .bearer_auth(self.token.as_str())
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        let body: ApiSuccessAsync<Vec<PeerStatusAsync>> = resp.json().await?;
        Ok(body.data)
    }

    async fn update_profile(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserInfoAsync, Self::Error> {
        let resp = self
            .client
            .put(format!("{}/profile/update", self.base_url))
            .bearer_auth(self.token.as_str())
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        let body: ProfileUpdateRespAsync = resp.json().await?;
        Ok(body.user)
    }

    async fn delete_account(&self) -> Result<(), Self::Error> {
        let resp = self
            .client
            .delete(format!("{}/profile/delete", self.base_url))
            .bearer_auth(self.token.as_str())
            .send()
            .await?;
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(Self::parse_error(&text));
        }
        Ok(())
    }
}
