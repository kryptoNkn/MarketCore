use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::AppError,
    models::auth::{AuthResponse, LoginRequest, RegisterRequest},
    services,
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    request.validate().map_err(AppError::InvalidRequest)?;
    let access_token = services::register(&state, &request.email, &request.password).await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token,
            token_type: "Bearer",
            expires_in: state.auth.jwt_ttl_seconds,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    request.validate().map_err(AppError::InvalidRequest)?;
    let access_token = services::login(&state, &request.email, &request.password).await?;
    Ok(Json(AuthResponse {
        access_token,
        token_type: "Bearer",
        expires_in: state.auth.jwt_ttl_seconds,
    }))
}