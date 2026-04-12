//! Serveur de développement — simule l'API VPN pour les tests d'intégration.
//!
//! Démarre sur le port 8080 (ou $PORT).
//! Données préconfigurées : alice/pass123, bob/hunter2

use std::{
    collections::HashMap,
    sync::{atomic::{AtomicU64, Ordering}, Arc, Mutex},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types (redéfinis ici — le dev-server est standalone)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserInfo {
    id: u64,
    username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Server {
    id: u64,
    name: String,
    country: String,
    ip: String,
    public_key: String,
    listen_port: u16,
    subnet: String,
    is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectionInfo {
    peer_ip: String,
    config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerStatus {
    id: u64,
    user_id: u64,
    server_id: u64,
    public_key: String,
    allowed_ip: String,
    server: Server,
}

// ---------------------------------------------------------------------------
// Formats de réponse
// ---------------------------------------------------------------------------

fn ok<T: Serialize>(data: T) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(serde_json::json!({ "data": data })))
}

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": msg })))
}

// ---------------------------------------------------------------------------
// État en mémoire
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct UserRecord {
    id: u64,
    username: String,
    password: String,
}

#[derive(Debug)]
struct AppState {
    users: Vec<UserRecord>,
    tokens: HashMap<String, u64>,      // token -> user_id
    connections: HashMap<u64, u64>,    // user_id -> server_id
    next_user_id: u64,
}

impl AppState {
    fn new() -> Self {
        Self {
            users: vec![
                UserRecord { id: 1, username: "alice".into(), password: "pass123".into() },
                UserRecord { id: 2, username: "bob".into(), password: "hunter2".into() },
            ],
            tokens: HashMap::new(),
            connections: HashMap::new(),
            next_user_id: 3,
        }
    }

    fn make_token(username: &str) -> String {
        let id = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("dev-token-{}-{}", username, id)
    }

    fn user_id_from_token(&self, token: &str) -> Option<u64> {
        self.tokens.get(token).copied()
    }
}

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

type SharedState = Arc<Mutex<AppState>>;

// ---------------------------------------------------------------------------
// Données serveurs fictifs
// ---------------------------------------------------------------------------

fn mock_servers() -> Vec<Server> {
    vec![
        Server {
            id: 1,
            name: "Paris-01".into(),
            country: "FR".into(),
            ip: "10.0.0.1".into(),
            public_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
            listen_port: 51820,
            subnet: "10.8.0.0/24".into(),
            is_active: true,
        },
        Server {
            id: 2,
            name: "Berlin-01".into(),
            country: "DE".into(),
            ip: "10.0.0.2".into(),
            public_key: "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=".into(),
            listen_port: 51820,
            subnet: "10.9.0.0/24".into(),
            is_active: true,
        },
    ]
}

fn wg_config(server: &Server, user_id: u64) -> String {
    let prefix: String = server.subnet.split('.').take(3).collect::<Vec<_>>().join(".");
    format!(
        "[Interface]\nPrivateKey = CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=\nAddress = {prefix}.{user_id}/32\nDNS = 1.1.1.1\n\n[Peer]\nPublicKey = {pubkey}\nEndpoint = {ip}:{port}\nAllowedIPs = 0.0.0.0/0\nPersistentKeepalive = 25\n",
        prefix = prefix,
        user_id = user_id,
        pubkey = server.public_key,
        ip = server.ip,
        port = server.listen_port,
    )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

async fn login(
    State(state): State<SharedState>,
    Json(body): Json<Credentials>,
) -> impl IntoResponse {
    let mut s = state.lock().unwrap();
    // Trouve l'utilisateur et copie les données avant d'emprunter s en mutable
    let found = s
        .users
        .iter()
        .find(|u| u.username == body.username && u.password == body.password)
        .cloned();
    match found {
        None => err(StatusCode::UNAUTHORIZED, "invalid credentials"),
        Some(u) => {
            let token = AppState::make_token(&u.username);
            s.tokens.insert(token.clone(), u.id);
            ok(serde_json::json!({ "token": token, "user": UserInfo { id: u.id, username: u.username } }))
        }
    }
}

async fn register(
    State(state): State<SharedState>,
    Json(body): Json<Credentials>,
) -> impl IntoResponse {
    let mut s = state.lock().unwrap();
    if s.users.iter().any(|u| u.username == body.username) {
        return err(StatusCode::CONFLICT, "user already exists");
    }
    let id = s.next_user_id;
    s.next_user_id += 1;
    s.users.push(UserRecord { id, username: body.username.clone(), password: body.password });
    let token = AppState::make_token(&body.username);
    s.tokens.insert(token.clone(), id);
    ok(serde_json::json!({ "token": token, "user": UserInfo { id, username: body.username } }))
}

async fn logout(
    State(state): State<SharedState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let mut s = state.lock().unwrap();
    match s.tokens.remove(auth.token()) {
        Some(_) => ok(serde_json::Value::Null),
        None => err(StatusCode::UNAUTHORIZED, "invalid token"),
    }
}

async fn list_servers(
    State(state): State<SharedState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let s = state.lock().unwrap();
    if s.user_id_from_token(auth.token()).is_none() {
        return err(StatusCode::UNAUTHORIZED, "invalid token");
    }
    ok(mock_servers())
}

#[derive(Deserialize)]
struct ServerIdRequest {
    server_id: u64,
}

async fn connect(
    State(state): State<SharedState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(body): Json<ServerIdRequest>,
) -> impl IntoResponse {
    let mut s = state.lock().unwrap();
    let user_id = match s.user_id_from_token(auth.token()) {
        Some(id) => id,
        None => return err(StatusCode::UNAUTHORIZED, "invalid token"),
    };
    let servers = mock_servers();
    let server = match servers.iter().find(|sv| sv.id == body.server_id) {
        Some(sv) => sv.clone(),
        None => return err(StatusCode::NOT_FOUND, "server not found"),
    };
    let prefix: String = server.subnet.split('.').take(3).collect::<Vec<_>>().join(".");
    let peer_ip = format!("{}.{}", prefix, user_id);
    let config = wg_config(&server, user_id);
    s.connections.insert(user_id, body.server_id);
    ok(ConnectionInfo { peer_ip, config })
}

async fn disconnect(
    State(state): State<SharedState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(body): Json<ServerIdRequest>,
) -> impl IntoResponse {
    let mut s = state.lock().unwrap();
    let user_id = match s.user_id_from_token(auth.token()) {
        Some(id) => id,
        None => return err(StatusCode::UNAUTHORIZED, "invalid token"),
    };
    if !mock_servers().iter().any(|sv| sv.id == body.server_id) {
        return err(StatusCode::NOT_FOUND, "server not found");
    }
    s.connections.remove(&user_id);
    ok(serde_json::Value::Null)
}

async fn peer_status(
    State(state): State<SharedState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let s = state.lock().unwrap();
    let user_id = match s.user_id_from_token(auth.token()) {
        Some(id) => id,
        None => return err(StatusCode::UNAUTHORIZED, "invalid token"),
    };
    let servers = mock_servers();
    let statuses: Vec<PeerStatus> = s
        .connections
        .iter()
        .filter(|(&uid, _)| uid == user_id)
        .filter_map(|(&uid, &srv_id)| {
            let server = servers.iter().find(|sv| sv.id == srv_id)?.clone();
            Some(PeerStatus {
                id: uid,
                user_id: uid,
                server_id: srv_id,
                public_key: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=".into(),
                allowed_ip: "0.0.0.0/0".into(),
                server,
            })
        })
        .collect();
    ok(statuses)
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    username: String,
    password: String,
}

async fn update_profile(
    State(state): State<SharedState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(body): Json<UpdateProfileRequest>,
) -> impl IntoResponse {
    let mut s = state.lock().unwrap();
    let user_id = match s.user_id_from_token(auth.token()) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid token"}))),
    };
    if let Some(u) = s.users.iter_mut().find(|u| u.id == user_id) {
        u.username = body.username.clone();
        u.password = body.password;
    }
    let info = UserInfo { id: user_id, username: body.username };
    (StatusCode::OK, Json(serde_json::json!({ "user": info })))
}

async fn delete_account(
    State(state): State<SharedState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let mut s = state.lock().unwrap();
    let user_id = match s.user_id_from_token(auth.token()) {
        Some(id) => id,
        None => return err(StatusCode::UNAUTHORIZED, "invalid token"),
    };
    s.users.retain(|u| u.id != user_id);
    s.tokens.retain(|_, &mut uid| uid != user_id);
    s.connections.remove(&user_id);
    ok(serde_json::Value::Null)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let state: SharedState = Arc::new(Mutex::new(AppState::new()));

    let app = Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/logout", post(logout))
        .route("/vpn/servers", get(list_servers))
        .route("/vpn/connect", post(connect))
        .route("/vpn/disconnect", post(disconnect))
        .route("/vpn/status", get(peer_status))
        .route("/profile/update", put(update_profile))
        .route("/profile/delete", delete(delete_account))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("dev-server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
