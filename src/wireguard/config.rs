use super::*;
impl WireGuardConfig {
    /// Parse une config WireGuard au format INI retournée par l'API.
    pub fn parse(config_str: &str) -> Result<Self, WireGuardError> {
        let mut private_key = None;
        let mut address = None;
        let mut dns = None;
        let mut peer_public_key = None;
        let mut endpoint = None;
        let mut allowed_ips = None;
        let mut persistent_keepalive = 25u16;

        for line in config_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('[') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "PrivateKey" => private_key = Some(value.to_string()),
                    "Address" => address = Some(value.to_string()),
                    "DNS" => dns = Some(value.to_string()),
                    "PublicKey" => peer_public_key = Some(value.to_string()),
                    "Endpoint" => endpoint = Some(value.to_string()),
                    "AllowedIPs" => allowed_ips = Some(value.to_string()),
                    "PersistentKeepalive" => {
                        persistent_keepalive = value.parse().unwrap_or(25);
                    }
                    _ => {}
                }
            }
        }

        Ok(WireGuardConfig {
            private_key: private_key
                .ok_or_else(|| WireGuardError::MissingField("PrivateKey".into()))?,
            address: address.ok_or_else(|| WireGuardError::MissingField("Address".into()))?,
            dns: dns.ok_or_else(|| WireGuardError::MissingField("DNS".into()))?,
            peer_public_key: peer_public_key
                .ok_or_else(|| WireGuardError::MissingField("PublicKey".into()))?,
            endpoint: endpoint.ok_or_else(|| WireGuardError::MissingField("Endpoint".into()))?,
            allowed_ips: allowed_ips
                .ok_or_else(|| WireGuardError::MissingField("AllowedIPs".into()))?,
            persistent_keepalive,
        })
    }

    /// Sérialise la config en format INI WireGuard standard.
    pub fn to_ini(&self) -> String {
        format!(
            "[Interface]\n\
             PrivateKey = {}\n\
             Address = {}\n\
             DNS = {}\n\
             \n\
             [Peer]\n\
             PublicKey = {}\n\
             Endpoint = {}\n\
             AllowedIPs = {}\n\
             PersistentKeepalive = {}\n",
            self.private_key,
            self.address,
            self.dns,
            self.peer_public_key,
            self.endpoint,
            self.allowed_ips,
            self.persistent_keepalive,
        )
    }
}
