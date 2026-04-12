use super::*;

pub fn register(base_url: &str, username: &str, password: &str) -> Result<AuthResponse, AuthError> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()?;

    if !resp.status().is_success() {
        let err: ApiError = resp.json().unwrap_or(ApiError {
            error: "unknown error".into(),
        });
        return Err(AuthError::Api(err.error));
    }

    let success: ApiSuccess = resp.json()?;
    Ok(AuthResponse {
        token: success.data.token,
        user: success.data.user,
    })
}

pub fn login(base_url: &str, username: &str, password: &str) -> Result<AuthResponse, AuthError> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()?;

    if !resp.status().is_success() {
        let err: ApiError = resp.json().unwrap_or(ApiError {
            error: "unknown error".into(),
        });
        return Err(AuthError::Api(err.error));
    }

    let success: ApiSuccess = resp.json()?;
    Ok(AuthResponse {
        token: success.data.token,
        user: success.data.user,
    })
}

pub fn logout(base_url: &str, token: &str) -> Result<(), AuthError> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/auth/logout", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    if !resp.status().is_success() {
        let err: ApiError = resp.json().unwrap_or(ApiError {
            error: "unknown error".into(),
        });
        return Err(AuthError::Api(err.error));
    }

    Ok(())
}
