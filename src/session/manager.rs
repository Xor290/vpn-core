use crate::backend::backend::{Server, UserInfo, VpnBackend};
use crate::wireguard::WireGuardConfig;

use super::{Session, SessionError};

impl<B: VpnBackend> Session<B> {
    pub fn login(
        backend: B,
        username: &str,
        password: &str,
    ) -> Result<Self, SessionError<B::Error>> {
        let auth = backend.login(username, password).map_err(SessionError::Backend)?;
        Ok(Self {
            backend,
            token: auth.token,
            user: auth.user,
            current_server: None,
            config: None,
        })
    }

    pub fn register(
        backend: B,
        username: &str,
        password: &str,
    ) -> Result<Self, SessionError<B::Error>> {
        let auth = backend.register(username, password).map_err(SessionError::Backend)?;
        Ok(Self {
            backend,
            token: auth.token,
            user: auth.user,
            current_server: None,
            config: None,
        })
    }

    pub fn user(&self) -> &UserInfo {
        &self.user
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn current_server(&self) -> Option<&Server> {
        self.current_server.as_ref()
    }

    pub fn current_config(&self) -> Option<&WireGuardConfig> {
        self.config.as_ref()
    }

    pub fn is_connected(&self) -> bool {
        self.current_server.is_some()
    }

    pub fn list_servers(&self) -> Result<Vec<Server>, SessionError<B::Error>> {
        self.backend.list_servers().map_err(SessionError::Backend)
    }

    pub fn connect(&mut self, server_id: u64) -> Result<&WireGuardConfig, SessionError<B::Error>> {
        let conn = self.backend.connect(server_id).map_err(SessionError::Backend)?;
        let wg_config = WireGuardConfig::parse(&conn.config)?;

        let servers = self.backend.list_servers().map_err(SessionError::Backend)?;
        self.current_server = servers.into_iter().find(|s| s.id == server_id);
        self.config = Some(wg_config);

        Ok(self.config.as_ref().unwrap())
    }

    pub fn disconnect(&mut self) -> Result<(), SessionError<B::Error>> {
        let server = self.current_server.as_ref().ok_or(SessionError::NotConnected)?;
        let server_id = server.id;
        self.backend.disconnect(server_id).map_err(SessionError::Backend)?;
        self.current_server = None;
        self.config = None;
        Ok(())
    }

    pub fn switch_server(
        &mut self,
        new_server_id: u64,
    ) -> Result<&WireGuardConfig, SessionError<B::Error>> {
        if self.is_connected() {
            self.disconnect()?;
        }
        self.connect(new_server_id)
    }

    pub fn update_profile(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(), SessionError<B::Error>> {
        self.user = self.backend.update_profile(username, password).map_err(SessionError::Backend)?;
        Ok(())
    }

    pub fn delete_account(&mut self) -> Result<(), SessionError<B::Error>> {
        self.backend.delete_account().map_err(SessionError::Backend)?;
        self.current_server = None;
        self.config = None;
        Ok(())
    }

    pub fn logout(self) -> Result<(), SessionError<B::Error>> {
        self.backend.logout(&self.token).map_err(SessionError::Backend)
    }
}
