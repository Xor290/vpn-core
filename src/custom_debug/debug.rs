use crate::backend::core::AuthResponse;
use crate::wireguard::WireGuardConfig;
use std::fmt;

impl fmt::Debug for WireGuardConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WireGuardConfig")
            .field("private_key", &"[REDACTED]")
            .field("address", &self.address)
            .field("dns", &self.dns)
            .field("peer_public_key", &self.peer_public_key)
            .field("endpoint", &self.endpoint)
            .field("allowed_ips", &self.allowed_ips)
            .field("persistent_keepalive", &self.persistent_keepalive)
            .finish()
    }
}

impl fmt::Debug for AuthResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthResponse")
            .field("token", &"[REDACTED]")
            .field("user", &self.user)
            .finish()
    }
}
