use std::io;

use crate::config::ConfigError;

#[derive(Debug)]
pub enum AppError {
    Config(ConfigError),
    Database(sqlx::Error),
    InvalidServerAddress,
    ServerBind(io::Error),
    Server(io::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(f, "configuration error: {error}"),
            Self::Database(error) => write!(f, "database error: {error}"),
            Self::InvalidServerAddress => write!(f, "invalid server address"),
            Self::ServerBind(error) => write!(f, "failed to bind server: {error}"),
            Self::Server(error) => write!(f, "server error: {error}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
