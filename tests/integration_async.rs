#[cfg(feature = "testing-async")]
mod integration_async {
    use vpn_core::backend::http_async::HttpAsyncBackend;
    use vpn_core::AsyncSession;

    fn server_url() -> String {
        std::env::var("VPN_TEST_URL").unwrap_or_else(|_| "http://localhost:8080".into())
    }

    fn backend() -> HttpAsyncBackend {
        HttpAsyncBackend::new_insecure(&server_url(), "")
    }

    #[tokio::test]
    async fn test_register() {
        let username = format!(
            "async_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        );
        let session = AsyncSession::register(backend(), &username, "pass1234")
            .await
            .expect("register doit réussir");
        session.logout().await.expect("logout doit réussir");
    }
    #[tokio::test]
    async fn test_login() {
        let session = AsyncSession::login(backend(), "alice", "pass123")
            .await
            .expect("login doit réussir");
        session.logout().await.expect("logout doit réussir");
    }
}
