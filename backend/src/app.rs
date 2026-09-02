use axum::{Router, routing::get};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
    .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
