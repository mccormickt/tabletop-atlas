use serde::Deserialize;
use std::sync::OnceLock;

use super::config::AuthConfig;

static OIDC_CLIENT: OnceLock<OidcClient> = OnceLock::new();

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

pub struct OidcClient {
    http_client: reqwest::Client,
}

impl OidcClient {
    pub fn init() -> Result<&'static Self, String> {
        if OIDC_CLIENT.get().is_some() {
            return Ok(OIDC_CLIENT.get().unwrap());
        }

        let client = Self {
            http_client: reqwest::Client::new(),
        };

        OIDC_CLIENT
            .set(client)
            .map_err(|_| "OIDC client already initialized".to_string())?;

        Ok(OIDC_CLIENT.get().unwrap())
    }

    /// Get the OIDC client. Panics if not initialized.
    /// Use this for internal code that runs after initialization.
    #[allow(dead_code)]
    pub fn get() -> &'static Self {
        OIDC_CLIENT
            .get()
            .expect("OidcClient not initialized - ensure OidcClient::init() is called at startup")
    }

    /// Get the OIDC client if initialized, returning None otherwise.
    /// Use this in HTTP handlers to return proper errors instead of panicking.
    pub fn try_get() -> Option<&'static Self> {
        OIDC_CLIENT.get()
    }

    pub fn generate_auth_url(&self) -> (String, String, String) {
        let config = AuthConfig::get();
        let state = uuid::Uuid::new_v4().to_string();
        let nonce = uuid::Uuid::new_v4().to_string();

        let auth_url = format!(
            "https://accounts.google.com/o/oauth2/v2/auth?\
            client_id={}&\
            redirect_uri={}&\
            response_type=code&\
            scope=openid%20email%20profile&\
            state={}&\
            nonce={}",
            urlencoding::encode(&config.google_client_id),
            urlencoding::encode(&config.google_redirect_uri),
            state,
            nonce
        );

        (auth_url, state, nonce)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<GoogleTokenResponse, String> {
        let config = AuthConfig::get();

        let params = [
            ("code", code),
            ("client_id", &config.google_client_id),
            ("client_secret", &config.google_client_secret),
            ("redirect_uri", &config.google_redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token exchange request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: {}", error_text));
        }

        response
            .json::<GoogleTokenResponse>()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo, String> {
        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("User info request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("User info request failed: {}", error_text));
        }

        response
            .json::<GoogleUserInfo>()
            .await
            .map_err(|e| format!("Failed to parse user info: {}", e))
    }
}
