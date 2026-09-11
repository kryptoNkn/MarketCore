use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::header;
use uuid::Uuid;

use crate::error::AppError;
use crate::services;
use crate::state::AppState;

pub struct AuthenticatedUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let id = services::user_id_from_token(state, token)?;
        Ok(Self { id })
    }
}