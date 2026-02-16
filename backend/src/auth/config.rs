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
    /// Extract resolved values from clap matches.
    /// Safe to unwrap — clap enforces required args and defaults.
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        let jwt_access_expiry: i64 = matches
            .get_one::<String>("jwt-access-expiry")
            .unwrap()
            .parse()
            .expect("jwt-access-expiry must be a valid number");
        let jwt_refresh_expiry: i64 = matches
            .get_one::<String>("jwt-refresh-expiry")
            .unwrap()
            .parse()
            .expect("jwt-refresh-expiry must be a valid number");

        Self {
            google_client_id: matches
                .get_one::<String>("google-client-id")
                .unwrap()
                .clone(),
            google_client_secret: matches
                .get_one::<String>("google-client-secret")
                .unwrap()
                .clone(),
            google_redirect_uri: matches
                .get_one::<String>("google-redirect-uri")
                .unwrap()
                .clone(),
            jwt_secret: matches.get_one::<String>("jwt-secret").unwrap().clone(),
            jwt_access_expiry,
            jwt_refresh_expiry,
            frontend_url: matches.get_one::<String>("frontend-url").unwrap().clone(),
        }
    }

    pub fn init_with(config: AuthConfig) -> Result<&'static Self, String> {
        if AUTH_CONFIG.get().is_some() {
            return Ok(AUTH_CONFIG.get().unwrap());
        }

        AUTH_CONFIG
            .set(config)
            .map_err(|_| "AuthConfig already initialized".to_string())?;

        Ok(AUTH_CONFIG.get().unwrap())
    }

    /// Get the auth config. Panics if not initialized.
    /// Use this for internal code that runs after initialization.
    pub fn get() -> &'static Self {
        AUTH_CONFIG
            .get()
            .expect("AuthConfig not initialized - ensure AuthConfig::init() is called at startup")
    }

    /// Get the auth config if initialized, returning None otherwise.
    /// Use this in HTTP handlers to return proper errors instead of panicking.
    pub fn try_get() -> Option<&'static Self> {
        AUTH_CONFIG.get()
    }
}
