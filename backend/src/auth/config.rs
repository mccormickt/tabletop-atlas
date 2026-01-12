use std::sync::OnceLock;

static AUTH_CONFIG: OnceLock<AuthConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub jwt_secret: String,
    pub jwt_access_expiry: i64,
    pub jwt_refresh_expiry: i64,
    pub frontend_url: String,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| "GOOGLE_CLIENT_ID must be set")?;
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| "GOOGLE_CLIENT_SECRET must be set")?;
        let google_redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8080/api/auth/callback".to_string());
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                use rand::Rng;
                let secret: String = rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect();
                tracing::warn!("JWT_SECRET not set, using randomly generated secret. Set JWT_SECRET for production.");
                secret
            });
        let jwt_access_expiry: i64 = std::env::var("JWT_ACCESS_EXPIRY")
            .unwrap_or_else(|_| "900".to_string())
            .parse()
            .map_err(|_| "JWT_ACCESS_EXPIRY must be a number")?;
        let jwt_refresh_expiry: i64 = std::env::var("JWT_REFRESH_EXPIRY")
            .unwrap_or_else(|_| "604800".to_string())
            .parse()
            .map_err(|_| "JWT_REFRESH_EXPIRY must be a number")?;
        let frontend_url = std::env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        Ok(Self {
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            jwt_secret,
            jwt_access_expiry,
            jwt_refresh_expiry,
            frontend_url,
        })
    }

    pub fn init() -> Result<&'static Self, String> {
        if AUTH_CONFIG.get().is_some() {
            return Ok(AUTH_CONFIG.get().unwrap());
        }

        let config = Self::from_env()?;
        AUTH_CONFIG
            .set(config)
            .map_err(|_| "AuthConfig already initialized".to_string())?;

        Ok(AUTH_CONFIG.get().unwrap())
    }

    pub fn get() -> &'static Self {
        AUTH_CONFIG.get().expect("AuthConfig not initialized")
    }
}
