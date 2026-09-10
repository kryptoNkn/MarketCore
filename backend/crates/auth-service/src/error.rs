use std::io;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::ConfigError;
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Config(ConfigError),
    Database(sqlx::Error),
    Migrations(sqlx::migrate::MigrateError),
    InvalidServerAddress,
    ServerBind(io::Error),
    Server(io::Error),
    InvalidRequest(&'static str),
    InvalidCredentials,
    Conflict,
    Internal,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "configuration error: {error}"),
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::Migrations(error) => write!(formatter, "database migration error: {error}"),
            Self::InvalidServerAddress => formatter.write_str("invalid server address"),
            Self::ServerBind(error) => write!(formatter, "failed to bind server: {error}"),
            Self::Server(error) => write!(formatter, "server error: {error}"),
            Self::InvalidRequest(error) => write!(formatter, "invalid request: {error}"),
            Self::InvalidCredentials => formatter.write_str("invalid credentials"),
            Self::Conflict => formatter.write_str("resource already exists"),
            Self::Internal => formatter.write_str("internal server error"),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if matches!(error, sqlx::Error::Database(ref db) if db.constraint() == Some("users_email_key"))
        {
            Self::Conflict
        } else {
            tracing::error!(error = ?error, "database operation failed");
            Self::Internal
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::InvalidRequest(message) => (StatusCode::BAD_REQUEST, "invalid_request", message),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "invalid credentials",
            ),
            Self::Conflict => (StatusCode::CONFLICT, "conflict", "resource already exists"),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "internal server error",
            ),
            error => {
                tracing::error!(error = %error, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal server error",
                )
            }
        };
        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

impl std::error::Error for AppError {}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
