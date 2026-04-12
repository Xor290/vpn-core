# vpn-core

> Modular Rust library for building WireGuard-based VPN clients — bring your own backend, batteries included with HTTP.

Contient toute la logique métier : authentification, gestion de session, parsing des configs WireGuard. Le backend (HTTP, gRPC, mock...) est interchangeable via un trait.

## Ce que vpn-core fait

- Authentification (register, login, logout)
- Gestion de session : connexion, déconnexion, switch de serveur, profil
- Parsing et sérialisation des configs WireGuard (format INI)
- Backend HTTP inclus (feature `http-backend`, activé par défaut)
- Supprime les secret en mémoire au moment d'un drop

## Ce que vpn-core NE fait PAS

- Pas de manipulation directe du tunnel VPN (pas de wg-quick, pas de tun)
- Pas d'UI
- Pas de code natif Android

Le code appelant reçoit la config WireGuard parsée et l'applique selon la plateforme :
- **Desktop** : `wg-quick` / `wireguard.exe`
- **Android** : `VpnService` + WireGuard SDK

## Structure

```
vpn-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── backend/
    │   ├── mod.rs          # Re-exports publics
    │   ├── backend.rs      # Trait VpnBackend + types communs
    │   └── http.rs         # Implémentation HTTP (feature: http-backend)
    ├── custom_debug/
    │   ├── mod.rs
    │   └── debug.rs        # Impls Debug custom (secrets redactés)
    ├── session/
    │   ├── mod.rs          # Session<B> + SessionError<E>
    │   └── manager.rs      # login, connect, disconnect, switch_server...
    └── wireguard/
        ├── mod.rs
        └── config.rs       # WireGuardConfig parse/sérialise
```

## Dépendances

| Crate | Rôle | Optionnel |
|---|---|---|
| `reqwest` | Client HTTP | Oui (`http-backend`) |
| `serde` | Sérialisation | Non |
| `serde_json` | JSON | Oui (`http-backend`) |
| `thiserror` | Gestion d'erreurs | Non |
| `zeroize` | Zéroïsation mémoire des secrets | Non |

## Installation

```toml
# Avec le backend HTTP (par défaut)
vpn-core = "0.1"

# Sans le backend HTTP (backend custom)
vpn-core = { version = "0.1", default-features = false }
```

## Utilisation

### Avec le backend HTTP inclus

```rust
use vpn_core::{HttpSession, backend::HttpBackend};

// Créer le backend avec l'URL de l'API
let backend = HttpBackend::new("https://api.example.com", "");

// Login — le token est géré en interne
let mut session = HttpSession::login(backend, "alice", "secret")?;

// Lister les serveurs
let servers = session.list_servers()?;

// Se connecter — retourne la config WireGuard prête à appliquer
let config = session.connect(servers[0].id)?;
println!("{}", config.to_ini()?);

// Changer de serveur
let new_config = session.switch_server(servers[1].id)?;

// Déconnexion
session.disconnect()?;
session.logout()?;
```

### Avec un backend custom

Implémenter le trait `VpnBackend` suffit :

```rust
use vpn_core::backend::{VpnBackend, AuthResponse, Server, ConnectionInfo, PeerStatus, UserInfo};

struct MyBackend { /* ... */ }

impl VpnBackend for MyBackend {
    type Error = MyError;

    fn login(&self, username: &str, password: &str) -> Result<AuthResponse, Self::Error> {
        // appel gRPC, socket, mock...
    }

    fn register(&self, username: &str, password: &str) -> Result<AuthResponse, Self::Error> { /* ... */ }
    fn logout(&self, token: &str) -> Result<(), Self::Error> { /* ... */ }
    fn list_servers(&self) -> Result<Vec<Server>, Self::Error> { /* ... */ }
    fn connect(&self, server_id: u64) -> Result<ConnectionInfo, Self::Error> { /* ... */ }
    fn disconnect(&self, server_id: u64) -> Result<(), Self::Error> { /* ... */ }
    fn peer_status(&self) -> Result<Vec<PeerStatus>, Self::Error> { /* ... */ }
    fn update_profile(&self, username: &str, password: &str) -> Result<UserInfo, Self::Error> { /* ... */ }
    fn delete_account(&self) -> Result<(), Self::Error> { /* ... */ }
}

// Puis :
let mut session = Session::login(MyBackend::new(), "alice", "secret")?;
```

## Module `wireguard`

Parse et sérialise les configs WireGuard au format INI.

```rust
use vpn_core::wireguard::WireGuardConfig;

let config = WireGuardConfig::parse(raw_ini_str)?;

println!("{}", config.endpoint);      // "1.2.3.4:51820"

// Resérialise en INI standard (prêt pour wg-quick)
// Retourne Err(WireGuardError::InvalidFormat) si un champ contient un saut de ligne
let ini = config.to_ini()?;
```

## Sécurité

| Finding | Sévérité | Statut |
|---|---|---|
| Clé privée WireGuard et token auth affichés via `Debug` | MEDIUM | Corrigé — impls `Debug` custom dans `custom_debug/` |
| Token auth en mémoire non zéroïsé après usage | MEDIUM | Corrigé — `Zeroizing<String>` + `ZeroizeOnDrop` sur `Session` et `HttpBackend` |
| Injection INI via newline dans `to_ini()` | LOW | Corrigé — validation des champs avant sérialisation |
| `PersistentKeepalive` parsé sans validation de plage | MEDIUM | Corrigé — parse en `u16` (borné 0–65535 par le type) |

## Build

```bash
cargo build --lib

# Sans le backend HTTP
cargo build --lib --no-default-features
```

## Crate types

Le crate produit 3 sorties :

| Type | Usage |
|---|---|
| `lib` | Usage Rust natif (desktop) |
| `staticlib` | Linking statique C (mobile) |
| `cdylib` | FFI dynamique (mobile) |

## Correspondance API

| `VpnBackend` | Endpoint HTTP par défaut |
|---|---|
| `login()` | `POST /auth/login` |
| `register()` | `POST /auth/register` |
| `logout()` | `POST /auth/logout` |
| `list_servers()` | `GET /vpn/servers` |
| `connect()` | `POST /vpn/connect` |
| `disconnect()` | `POST /vpn/disconnect` |
| `peer_status()` | `GET /vpn/status` |
| `update_profile()` | `PUT /profile/update` |
| `delete_account()` | `DELETE /profile/delete` |
