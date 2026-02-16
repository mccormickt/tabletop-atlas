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

#[cfg(test)]
mod tests {
    use super::*;

    /// Initialize AuthConfig for tests using init_with().
    fn init_test_config() {
        let _ = AuthConfig::init_with(AuthConfig {
            google_client_id: String::new(),
            google_client_secret: String::new(),
            google_redirect_uri: String::new(),
            jwt_secret: "test-secret-key-for-jwt-tests".to_string(),
            jwt_access_expiry: 900,
            jwt_refresh_expiry: 604800,
            frontend_url: String::new(),
        });
    }

    #[test]
    fn test_access_token_roundtrip() {
        init_test_config();

        let token = create_access_token(42, "user@example.com", "user")
            .expect("should create access token");
        let claims = verify_access_token(&token).expect("should verify access token");

        assert_eq!(claims.sub, 42);
        assert_eq!(claims.email, "user@example.com");
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_refresh_token_roundtrip() {
        init_test_config();

        let token = create_refresh_token(42, "session-abc").expect("should create refresh token");
        let claims = verify_refresh_token(&token).expect("should verify refresh token");

        assert_eq!(claims.sub, 42);
        assert_eq!(claims.session_id, "session-abc");
    }
}
