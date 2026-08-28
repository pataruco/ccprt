use std::net::SocketAddr;
use std::sync::Arc;

use ccprt::{build_router, config::Config, AppState, GitHubClient};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("configuration error: {e}");
        std::process::exit(1);
    });

    let github = GitHubClient::new(config.app_id, &config.private_key, &config.github_api_url)
        .unwrap_or_else(|e| {
            eprintln!("github client error: {e}");
            std::process::exit(1);
        });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let state = AppState {
        config: Arc::new(config),
        github,
    };
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    tracing::info!("listening on {addr}");
    axum::serve(listener, build_router(state))
        .await
        .expect("server error");
}
