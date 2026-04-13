/// Tests d'intégration contre le dev-server Docker.
///
/// Prérequis :
///   docker compose up -d --wait
///
/// Lancer les tests :
///   cargo test --features testing --test integration
///
/// La variable d'environnement `VPN_TEST_URL` permet de pointer sur une autre instance
/// (défaut : http://localhost:8080).

#[cfg(feature = "testing")]
mod tests {
    use vpn_core::backend::HttpBackend;
    use vpn_core::session::Session;

    fn server_url() -> String {
        std::env::var("VPN_TEST_URL").unwrap_or_else(|_| "http://localhost:8080".into())
    }

    fn backend() -> HttpBackend {
        HttpBackend::new_insecure(&server_url(), "")
    }

    // -----------------------------------------------------------------------
    // Auth
    // -----------------------------------------------------------------------

    #[test]
    fn test_login_ok() {
        let session = Session::login(backend(), "alice", "pass123")
            .expect("login doit réussir avec des credentials valides");
        assert_eq!(session.user().username, "alice");
        session.logout().unwrap();
    }

    #[test]
    fn test_login_wrong_password() {
        let result = Session::login(backend(), "alice", "mauvais");
        assert!(
            result.is_err(),
            "login doit échouer avec un mauvais mot de passe"
        );
    }

    #[test]
    fn test_register_new_user() {
        // Utilise un nom unique pour éviter les conflits entre runs
        let username = format!(
            "user_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        );
        let session = Session::register(backend(), &username, "pwd")
            .expect("register doit réussir pour un nouvel utilisateur");
        assert_eq!(session.user().username, username);
        session.logout().unwrap();
    }

    #[test]
    fn test_register_duplicate_fails() {
        let result = Session::register(backend(), "alice", "n'importe");
        assert!(
            result.is_err(),
            "register doit échouer si l'utilisateur existe déjà"
        );
    }

    // -----------------------------------------------------------------------
    // Serveurs
    // -----------------------------------------------------------------------

    #[test]
    fn test_list_servers() {
        let session = Session::login(backend(), "alice", "pass123").unwrap();
        let servers = session
            .list_servers()
            .expect("list_servers doit retourner des serveurs");
        assert!(
            !servers.is_empty(),
            "la liste de serveurs ne doit pas être vide"
        );
        session.logout().unwrap();
    }

    // -----------------------------------------------------------------------
    // Connexion VPN
    // -----------------------------------------------------------------------

    #[test]
    fn test_connect_and_disconnect() {
        let mut session = Session::login(backend(), "bob", "hunter2").unwrap();

        let servers = session.list_servers().unwrap();
        let server_id = servers[0].id;

        let config = session
            .connect(server_id)
            .expect("connect doit retourner une config WireGuard");
        assert!(
            !config.endpoint.is_empty(),
            "l'endpoint WireGuard ne doit pas être vide"
        );
        assert!(session.is_connected());

        session.disconnect().expect("disconnect doit réussir");
        assert!(!session.is_connected());

        session.logout().unwrap();
    }

    #[test]
    fn test_connect_invalid_server() {
        let mut session = Session::login(backend(), "alice", "pass123").unwrap();
        let result = session.connect(9999);
        assert!(
            result.is_err(),
            "connect sur un serveur inexistant doit échouer"
        );
        session.logout().unwrap();
    }

    #[test]
    fn test_switch_server() {
        let mut session = Session::login(backend(), "alice", "pass123").unwrap();
        let servers = session.list_servers().unwrap();
        assert!(
            servers.len() >= 2,
            "il faut au moins 2 serveurs pour ce test"
        );

        session.connect(servers[0].id).unwrap();
        session
            .switch_server(servers[1].id)
            .expect("switch_server doit réussir");
        let current_id = session.current_server().unwrap().id;
        let endpoint = session.current_config().unwrap().endpoint.clone();
        assert_eq!(current_id, servers[1].id);
        assert!(!endpoint.is_empty());

        session.disconnect().unwrap();
        session.logout().unwrap();
    }

    // -----------------------------------------------------------------------
    // Peer status
    // -----------------------------------------------------------------------

    #[test]
    fn test_peer_status_while_connected() {
        // peer_status est appelé via le backend directement (pas exposé sur Session)
        use vpn_core::backend::VpnBackend;

        let b = backend();
        let auth = b.login("alice", "pass123").unwrap();
        let b_with_token = HttpBackend::new_insecure(&server_url(), &auth.token);

        b_with_token.connect(1).unwrap();
        let statuses = b_with_token
            .peer_status()
            .expect("peer_status doit retourner des données");
        assert!(
            !statuses.is_empty(),
            "au moins un peer doit être actif après connexion"
        );

        b_with_token.disconnect(1).unwrap();
        b_with_token.logout(&auth.token).unwrap();
    }

    // -----------------------------------------------------------------------
    // Profil
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_profile() {
        let username = format!(
            "upd_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        );
        let mut session = Session::register(backend(), &username, "old_pwd").unwrap();

        session
            .update_profile("new_name", "new_pwd")
            .expect("update_profile doit réussir");
        assert_eq!(session.user().username, "new_name");
        session.logout().unwrap();
    }

    #[test]
    fn test_delete_account() {
        let username = format!(
            "del_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        );
        let mut session = Session::register(backend(), &username, "pwd").unwrap();
        session
            .delete_account()
            .expect("delete_account doit réussir");
    }

    // -----------------------------------------------------------------------
    // WireGuard config
    // -----------------------------------------------------------------------

    #[test]
    fn test_wireguard_config_parseable() {
        let mut session = Session::login(backend(), "alice", "pass123").unwrap();
        let servers = session.list_servers().unwrap();
        let config = session.connect(servers[0].id).unwrap();

        // to_ini() valide que les champs ne contiennent pas de sauts de ligne
        let ini = config
            .to_ini()
            .expect("to_ini doit réussir sur une config valide");
        assert!(ini.contains("[Interface]"));
        assert!(ini.contains("[Peer]"));
        assert!(ini.contains("PrivateKey"));
        assert!(ini.contains("Endpoint"));

        session.disconnect().unwrap();
        session.logout().unwrap();
    }
}
