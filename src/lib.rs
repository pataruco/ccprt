pub mod config;
pub mod github;
pub mod payload;
pub mod signature;
pub mod validation;
pub mod webhook;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::config::Config;
pub use github::GitHubClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub github: GitHubClient,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", post(webhook::handle_webhook))
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
}
