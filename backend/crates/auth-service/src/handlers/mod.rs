use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::AppError,
    extract::AuthenticatedUser,
    models::auth::{
        AuthResponse, LoginRequest, LogoutRequest, MeResponse, RefreshRequest, RegisterRequest,
        normalize_email,
    },
    services,
    state::AppState,
};

fn tokens_to_response(state: &AppState, tokens: services::TokenPair) -> AuthResponse {
    AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: "Bearer",
        expires_in: state.auth.jwt_ttl_seconds,
        refresh_expires_in: state.auth.refresh_ttl_seconds,
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    request.validate().map_err(AppError::InvalidRequest)?;
    let email = normalize_email(&request.email);
    let tokens = services::register(&state, &email, &request.password).await?;
    Ok((StatusCode::CREATED, Json(tokens_to_response(&state, tokens))))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    request.validate().map_err(AppError::InvalidRequest)?;
    let email = normalize_email(&request.email);
    let tokens = services::login(&state, &email, &request.password).await?;
    Ok(Json(tokens_to_response(&state, tokens)))
}

pub async fn me(
    State(state): State<AppState>,
    AuthenticatedUser { id }: AuthenticatedUser,
) -> Result<Json<MeResponse>, AppError> {
    let profile = services::me(&state, id).await?;
    Ok(Json(MeResponse {
        id: profile.id,
        email: profile.email,
        role: profile.role,
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    request.validate().map_err(AppError::InvalidRequest)?;
    let tokens = services::refresh(&state, &request.refresh_token).await?;
    Ok(Json(tokens_to_response(&state, tokens)))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<StatusCode, AppError> {
    request.validate().map_err(AppError::InvalidRequest)?;
    services::logout(&state, &request.refresh_token).await?;
    Ok(StatusCode::NO_CONTENT)
}
