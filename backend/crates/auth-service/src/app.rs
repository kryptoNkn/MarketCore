use std::sync::Arc;
use std::time::Duration;
use axum::{
    Json, Router,
    http::{HeaderValue, Method, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use tower_governor::{
    GovernorLayer, errors::GovernorError, governor::GovernorConfigBuilder,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use crate::error::AppError;
use crate::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState, cors_origin: HeaderValue) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(cors_origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let rate_limit = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(20)
            .finish()
            .expect("rate limit configuration")
    );
    let limiter = rate_limit.limiter().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            limiter.retain_recent();
        }
    });

    let auth_routes = Router::new()
        .route("/v1/auth/register", post(handlers::register))
        .route("/v1/auth/login", post(handlers::login))
        .route("/v1/auth/refresh", post(handlers::refresh))
        .route("/v1/auth/logout", post(handlers::logout))
        .layer(
            GovernorLayer::new(rate_limit)
                .error_handler(rate_limit_response),
        );

    Router::new()
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/v1/auth/me", get(handlers::me))
        .merge(auth_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn rate_limit_response(error: GovernorError) -> axum::response::Response {
    match error {
        GovernorError::TooManyRequests {wait_time, ..} => {
            let mut response = AppError::RateLimited.into_response();
            if let Ok(value) = HeaderValue::from_str(&wait_time.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
            response
        }
        GovernorError::UnableToExtractKey | GovernorError::Other { .. } => {
            AppError::Internal.into_response()
        }
    }
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
) -> Result<Json<HealthResponse>, AppError> {
    sqlx::query("SELECT 1").execute(&db).await.map_err(|error| {
        tracing::error!(error = ?error, "readiness check failed");
        AppError::Unavailable
    })?;

    Ok(Json(HealthResponse {
        service: "auth-service",
        status: "ready",
    }))
}
