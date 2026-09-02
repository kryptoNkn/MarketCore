use std::io;

use common::ConfigError;

#[derive(Debug)]
pub enum AppError {
    Config(ConfigError),
    Database(sqlx::Error),
    Migrations(sqlx::migrate::MigrateError),
    InvalidServerAddress,
    ServerBind(io::Error),
    Server(io::Error),
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
        }
    }
}

impl std::error::Error for AppError {}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
