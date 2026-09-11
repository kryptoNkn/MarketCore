use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub password_repeat: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_email(&self.email)?;
        if self.password.len() < 12 || self.password.len() > 128 {
            return Err("password must be between 12 and 128 characters");
        }
        if self.password != self.password_repeat {
            return Err("passwords do not match");
        }
        Ok(())
    }
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.email.trim().is_empty() || self.password.is_empty() {
            return Err("email and password are required");
        }
        Ok(())
    }
}

impl RefreshRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_refresh_token(&self.refresh_token)
    }
}

impl LogoutRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_refresh_token(&self.refresh_token)
    }
}

fn validate_email(email: &str) -> Result<(), &'static str> {
    let email = normalize_email(email);
    let Some((local, domain)) = email.split_once('@') else {
        return Err("invalid email");
    };
    if local.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || email.len() > 320
        || email.contains(char::is_whitespace)
    {
        return Err("invalid email");
    }
    Ok(())
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_owned()
}

fn validate_refresh_token(token: &str) -> Result<(), &'static str> {
    if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("invalid refresh token");
    }
    Ok(())
}