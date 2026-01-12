use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use super::config::AuthConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user_id
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: i64, // user_id
    pub session_id: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_access_token(user_id: i64, email: &str, role: &str) -> Result<String, String> {
    let config = AuthConfig::get();
    let now = chrono::Utc::now().timestamp();
    let exp = now + config.jwt_access_expiry;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to create access token: {}", e))
}

pub fn create_refresh_token(user_id: i64, session_id: &str) -> Result<String, String> {
    let config = AuthConfig::get();
    let now = chrono::Utc::now().timestamp();
    let exp = now + config.jwt_refresh_expiry;

    let claims = RefreshClaims {
        sub: user_id,
        session_id: session_id.to_string(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to create refresh token: {}", e))
}

pub fn verify_access_token(token: &str) -> Result<Claims, String> {
    let config = AuthConfig::get();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Failed to verify access token: {}", e))
}

pub fn verify_refresh_token(token: &str) -> Result<RefreshClaims, String> {
    let config = AuthConfig::get();

    decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Failed to verify refresh token: {}", e))
}
