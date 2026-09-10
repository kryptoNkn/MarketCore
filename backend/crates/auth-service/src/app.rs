use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Serialize;

use crate::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/v1/auth/register", post(handlers::register))
        .route("/v1/auth/login", post(handlers::login))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
}

async fn liveness() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "auth-service",
        status: "ok",
    })
}

async fn readiness(
    axum::extract::State(AppState { db, .. }): axum::extract::State<AppState>,
) -> Result<Json<HealthResponse>, (axum::http::StatusCode, &'static str)> {
    sqlx::query("SELECT 1").execute(&db).await.map_err(|_| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "database unavailable",
        )
    })?;

    Ok(Json(HealthResponse {
        service: "auth-service",
        status: "ready",
    }))
}
