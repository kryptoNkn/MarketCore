use serde::{Deserialize, Serialize};

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
    pub token_type: &'static str,
    pub expires_in: u64,
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

fn validate_email(email: &str) -> Result<(), &'static str> {
    let email = email.trim();
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
