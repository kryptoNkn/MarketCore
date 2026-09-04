use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}