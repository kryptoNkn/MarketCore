use std::env;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub server_host: String,
    pub server_port: u16,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_owned())
            .parse()
            .map_err(|_| ConfigError::InvalidServerPort)?;
        Ok(Self {
            server_host,
            server_port,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;
        Ok(Self { database_url })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidServerPort,
    MissingDatabaseUrl,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidServerPort => {
                formatter.write_str("SERVER_PORT must be a valid port number")
            }
            Self::MissingDatabaseUrl => {
                formatter.write_str("DATABASE_URL environment variable is missing")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
