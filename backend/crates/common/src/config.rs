use std::env;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub server_host: String,
    pub server_port: u16,
    pub cors_origin: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_owned())
            .parse()
            .map_err(|_| ConfigError::InvalidServerPort)?;
        let cors_origin =
            env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_owned());
        Ok(Self {
            server_host,
            server_port,
            cors_origin,
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

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_ttl_seconds: u64,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::MissingJwtSecret)?;
        if jwt_secret.len() < 32 {
            return Err(ConfigError::WeakJwtSecret);
        }

        let jwt_ttl_seconds = env::var("JWT_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_owned())
            .parse()
            .map_err(|_| ConfigError::InvalidJwtTtl)?;

        Ok(Self {
            jwt_secret,
            jwt_ttl_seconds,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidServerPort,
    MissingDatabaseUrl,
    MissingJwtSecret,
    WeakJwtSecret,
    InvalidJwtTtl,
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
            Self::MissingJwtSecret => {
                formatter.write_str("JWT_SECRET environment variable is missing")
            }
            Self::WeakJwtSecret => formatter.write_str("JWT_SECRET must be at least 32 bytes"),
            Self::InvalidJwtTtl => formatter.write_str("JWT_TTL_SECONDS must be a valid number"),
        }
    }
}

impl std::error::Error for ConfigError {}
