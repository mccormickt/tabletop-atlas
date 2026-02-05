use cookie::{Cookie, SameSite};
use dropshot::{HttpError, HttpResponseHeaders, HttpResponseOk, Query, RequestContext, endpoint};
use http::header::{HeaderName, HeaderValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::{AuthConfig, OidcClient, extract_auth, require_auth};
use crate::db::{sessions, users};
use crate::models::{CreateUserRequest, UserInfo};

use super::{CorsHeaders, internal_error, not_found_error, success_response, unauthorized_error};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AuthResponse {
    pub user: UserInfo,
}

fn build_cookie(name: &str, value: &str, path: &str, max_age: i64, secure: bool) -> String {
    let mut cookie = Cookie::build((name, value))
        .path(path)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::seconds(max_age));

    if secure {
        cookie = cookie.secure(true);
    }

    cookie.build().to_string()
}

fn build_clear_cookie(name: &str, path: &str) -> String {
    Cookie::build((name, ""))
        .path(path)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::ZERO)
        .build()
        .to_string()
}

/// Initiate Google OAuth login - redirects to Google
#[endpoint {
    method = GET,
    path = "/api/auth/login",
    tags = ["auth"]
}]
pub async fn login(
    _rqctx: RequestContext<AppState>,
) -> Result<http::Response<dropshot::Body>, HttpError> {
    let config =
        AuthConfig::try_get().ok_or_else(|| internal_error("Auth not configured".to_string()))?;
    let oidc = OidcClient::try_get()
        .ok_or_else(|| internal_error("OIDC client not configured".to_string()))?;
    let (auth_url, state, _nonce) = oidc.generate_auth_url();

    // Store state in a secure cookie for CSRF validation
    let is_secure = config.frontend_url.starts_with("https");
    let state_cookie = build_cookie("oauth_state", &state, "/api/auth", 600, is_secure);

    let response = http::Response::builder()
        .status(http::StatusCode::FOUND)
        .header(http::header::LOCATION, auth_url)
        .header(http::header::SET_COOKIE, state_cookie)
        .body(dropshot::Body::empty())
        .map_err(|e| internal_error(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

fn extract_cookie(rqctx: &RequestContext<AppState>, name: &str) -> Option<String> {
    let request = &rqctx.request;
    let cookie_header = request.headers().get("cookie")?.to_str().ok()?;
    cookie_header
        .split(';')
        .filter_map(|c| {
            let mut parts = c.trim().splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(n), Some(v)) if n == name => Some(v.to_string()),
                _ => None,
            }
        })
        .next()
}

/// Handle Google OAuth callback
#[endpoint {
    method = GET,
    path = "/api/auth/callback",
    tags = ["auth"]
}]
pub async fn callback(
    rqctx: RequestContext<AppState>,
    query: Query<CallbackQuery>,
) -> Result<http::Response<dropshot::Body>, HttpError> {
    let config =
        AuthConfig::try_get().ok_or_else(|| internal_error("Auth not configured".to_string()))?;
    let oidc = OidcClient::try_get()
        .ok_or_else(|| internal_error("OIDC client not configured".to_string()))?;
    let db = rqctx.context().db();
    let query = query.into_inner();

    // Validate OAuth state to prevent CSRF attacks
    let stored_state = extract_cookie(&rqctx, "oauth_state")
        .ok_or_else(|| unauthorized_error("Missing OAuth state cookie".to_string()))?;
    let query_state = query
        .state
        .as_ref()
        .ok_or_else(|| unauthorized_error("Missing state parameter".to_string()))?;

    if stored_state != *query_state {
        return Err(unauthorized_error("OAuth state mismatch".to_string()));
    }

    // Exchange code for tokens
    let token_response = oidc
        .exchange_code(&query.code)
        .await
        .map_err(|e| internal_error(format!("Token exchange failed: {}", e)))?;

    // Get user info from Google
    let user_info = oidc
        .get_user_info(&token_response.access_token)
        .await
        .map_err(|e| internal_error(format!("Failed to get user info: {}", e)))?;

    // Find or create user
    let user = match users::find_by_google_sub(&db, &user_info.sub).await {
        Ok(Some(user)) => {
            // Update user info if changed
            users::update_user(
                &db,
                user.id,
                user_info.name.clone(),
                user_info.picture.clone(),
            )
            .await
            .map_err(|e| internal_error(format!("Failed to update user: {}", e)))?
            .unwrap_or(user)
        }
        Ok(None) => {
            // Create new user
            users::create_user(
                &db,
                CreateUserRequest {
                    google_sub: user_info.sub,
                    email: user_info.email.clone(),
                    display_name: user_info.name,
                    picture_url: user_info.picture,
                },
            )
            .await
            .map_err(|e| internal_error(format!("Failed to create user: {}", e)))?
        }
        Err(e) => return Err(internal_error(format!("Database error: {}", e))),
    };

    // Create tokens - generate session_id first, then refresh token, then session
    let access_token = crate::auth::create_access_token(user.id, &user.email, &user.role)
        .map_err(internal_error)?;

    let session_id = sessions::generate_session_id();
    let refresh_token =
        crate::auth::create_refresh_token(user.id, &session_id).map_err(internal_error)?;

    let refresh_expiry = chrono::Utc::now() + chrono::Duration::seconds(config.jwt_refresh_expiry);

    // Create session in DB with the refresh token hash
    sessions::create_session(&db, &session_id, user.id, &refresh_token, refresh_expiry)
        .await
        .map_err(|e| internal_error(format!("Failed to create session: {}", e)))?;

    // Build redirect response with cookies
    let is_secure = config.frontend_url.starts_with("https");
    let access_cookie = build_cookie(
        "access_token",
        &access_token,
        "/",
        config.jwt_access_expiry,
        is_secure,
    );
    let refresh_cookie = build_cookie(
        "refresh_token",
        &refresh_token,
        "/api/auth",
        config.jwt_refresh_expiry,
        is_secure,
    );
    // Clear oauth_state cookie after successful validation
    let clear_state_cookie = build_clear_cookie("oauth_state", "/api/auth");

    let redirect_url = format!("{}/", config.frontend_url);

    let refresh_cookie_header = HeaderValue::from_str(&refresh_cookie)
        .map_err(|e| internal_error(format!("Invalid refresh cookie value: {}", e)))?;
    let clear_state_header = HeaderValue::from_str(&clear_state_cookie)
        .map_err(|e| internal_error(format!("Invalid clear state cookie value: {}", e)))?;

    let response = http::Response::builder()
        .status(http::StatusCode::FOUND)
        .header(http::header::LOCATION, redirect_url)
        .header(http::header::SET_COOKIE, access_cookie)
        .header(HeaderName::from_static("set-cookie"), refresh_cookie_header)
        .header(HeaderName::from_static("set-cookie"), clear_state_header)
        .header("Access-Control-Allow-Origin", "*")
        .body(dropshot::Body::empty())
        .map_err(|e| internal_error(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// Get current user info
#[endpoint {
    method = GET,
    path = "/api/auth/me",
    tags = ["auth"]
}]
pub async fn get_me(
    rqctx: RequestContext<AppState>,
) -> Result<HttpResponseHeaders<HttpResponseOk<AuthResponse>, CorsHeaders>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();

    let full_user = users::get_user_by_id(&db, user.user_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("User not found".to_string()))?;

    success_response(AuthResponse {
        user: full_user.into(),
    })
}

/// Logout and clear cookies
#[endpoint {
    method = POST,
    path = "/api/auth/logout",
    tags = ["auth"]
}]
pub async fn logout(
    rqctx: RequestContext<AppState>,
) -> Result<http::Response<dropshot::Body>, HttpError> {
    // If user is authenticated, delete their session
    if let Some(user) = extract_auth(&rqctx) {
        let db = rqctx.context().db();
        if let Err(e) = sessions::delete_user_sessions(&db, user.user_id).await {
            slog::error!(rqctx.log, "Failed to delete user sessions during logout";
                "user_id" => user.user_id, "error" => %e);
        }
    }

    let access_cookie = build_clear_cookie("access_token", "/");
    let refresh_cookie = build_clear_cookie("refresh_token", "/api/auth");

    let refresh_cookie_header = HeaderValue::from_str(&refresh_cookie)
        .map_err(|e| internal_error(format!("Invalid refresh cookie value: {}", e)))?;

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::SET_COOKIE, access_cookie)
        .header(HeaderName::from_static("set-cookie"), refresh_cookie_header)
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Type", "application/json")
        .body(dropshot::Body::from("{\"success\": true}"))
        .map_err(|e| internal_error(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// Refresh access token
#[endpoint {
    method = POST,
    path = "/api/auth/refresh",
    tags = ["auth"]
}]
pub async fn refresh(
    rqctx: RequestContext<AppState>,
) -> Result<http::Response<dropshot::Body>, HttpError> {
    let config =
        AuthConfig::try_get().ok_or_else(|| internal_error("Auth not configured".to_string()))?;
    let db = rqctx.context().db();

    // Get refresh token from cookie
    let refresh_token = {
        let request = &rqctx.request;
        let cookie_header = request
            .headers()
            .get("cookie")
            .and_then(|h: &http::HeaderValue| h.to_str().ok())
            .unwrap_or("");

        cookie_header
            .split(';')
            .filter_map(|c: &str| {
                let mut parts = c.trim().splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some("refresh_token"), Some(v)) => Some(v.to_string()),
                    _ => None,
                }
            })
            .next()
    };

    let refresh_token =
        refresh_token.ok_or_else(|| unauthorized_error("No refresh token".to_string()))?;

    // Verify refresh token
    let claims = crate::auth::verify_refresh_token(&refresh_token)
        .map_err(|_| unauthorized_error("Invalid refresh token".to_string()))?;

    // Find valid session
    let session = sessions::find_valid_session(&db, &claims.session_id, &refresh_token)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| unauthorized_error("Session expired".to_string()))?;

    // Get user
    let user = users::get_user_by_id(&db, session.user_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| unauthorized_error("User not found".to_string()))?;

    // Create new access token
    let access_token = crate::auth::create_access_token(user.id, &user.email, &user.role)
        .map_err(internal_error)?;

    let is_secure = config.frontend_url.starts_with("https");
    let access_cookie = build_cookie(
        "access_token",
        &access_token,
        "/",
        config.jwt_access_expiry,
        is_secure,
    );

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::SET_COOKIE, access_cookie)
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Type", "application/json")
        .body(dropshot::Body::from("{\"success\": true}"))
        .map_err(|e| internal_error(format!("Failed to build response: {}", e)))?;

    Ok(response)
}
