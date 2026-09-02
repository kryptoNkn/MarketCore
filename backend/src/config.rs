use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_owned())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidServerPort)?;

        let database_url = 
            env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;

        Ok(Self {
            server_host,
            server_port,
            database_url,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidServerPort,
    MissingDatabaseUrl,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidServerPort => write!(f, "SERVER_PORT must be a valid port number"),
            Self::MissingDatabaseUrl =>  write!(f, "DATABASE_URL environment variable is missing")
        }
    }
}

impl std::error::Error for ConfigError {}
