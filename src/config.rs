use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types publics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    Http,
    Async,
    Grpc,
}

impl Default for BackendKind {
    fn default() -> Self {
        Self::Http
    }
}

impl std::fmt::Display for BackendKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::Async => write!(f, "async"),
            Self::Grpc => write!(f, "grpc"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    #[serde(default)]
    pub kind: BackendKind,
    #[serde(default)]
    pub url: String,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            kind: BackendKind::Http,
            url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VpnConfig {
    #[serde(default)]
    pub backend: BackendConfig,
}

// ---------------------------------------------------------------------------
// Erreur
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    NotFound,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "config I/O error: {}", e),
            Self::Parse(e) => write!(f, "config parse error: {}", e),
            Self::NotFound => write!(f, "no config file found"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e)
    }
}

// ---------------------------------------------------------------------------
// Chargement
// ---------------------------------------------------------------------------

impl VpnConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::resolve_path().ok_or(ConfigError::NotFound)?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    fn resolve_path() -> Option<PathBuf> {
        // 1. Variable d'environnement
        if let Ok(p) = std::env::var("VPN_CONFIG") {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }

        let local = PathBuf::from("vpn-core.toml");
        if local.exists() {
            return Some(local);
        }

        #[cfg(not(target_os = "windows"))]
        if let Ok(home) = std::env::var("HOME") {
            let xdg = PathBuf::from(home).join(".config/vpn-core/config.toml");
            if xdg.exists() {
                return Some(xdg);
            }
        }
        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            let win = PathBuf::from(appdata).join("vpn-core\\config.toml");
            if win.exists() {
                return Some(win);
            }
        }

        None
    }
}
