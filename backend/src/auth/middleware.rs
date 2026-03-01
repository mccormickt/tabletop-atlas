use cookie::Cookie;
use dropshot::{ClientErrorStatusCode, HttpError, RequestContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::jwt;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub email: String,
    pub role: String,
}

fn parse_cookies(cookie_header: &str) -> HashMap<String, String> {
    cookie_header
        .split(';')
        .filter_map(|c| Cookie::parse(c.trim()).ok())
        .map(|c| (c.name().to_string(), c.value().to_string()))
        .collect()
}

pub fn extract_auth(rqctx: &RequestContext<AppState>) -> Option<AuthenticatedUser> {
    let request = &rqctx.request;

    let cookie_header = match request.headers().get("cookie") {
        Some(h) => h,
        None => {
            slog::debug!(rqctx.log, "extract_auth: no cookie header");
            return None;
        }
    };

    let cookie_str = match cookie_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            slog::debug!(rqctx.log, "extract_auth: invalid UTF-8 in cookie header");
            return None;
        }
    };

    let cookies = parse_cookies(cookie_str);

    let access_token = match cookies.get("access_token") {
        Some(t) => t,
        None => {
            slog::debug!(rqctx.log, "extract_auth: no access_token cookie");
            return None;
        }
    };

    let claims = match jwt::verify_access_token(access_token) {
        Ok(c) => c,
        Err(e) => {
            slog::debug!(rqctx.log, "extract_auth: JWT verification failed"; "error" => %e);
            return None;
        }
    };

    Some(AuthenticatedUser {
        user_id: claims.sub,
        email: claims.email,
        role: claims.role,
    })
}

pub fn require_auth(rqctx: &RequestContext<AppState>) -> Result<AuthenticatedUser, HttpError> {
    extract_auth(rqctx).ok_or_else(|| {
        HttpError::for_client_error(
            None,
            ClientErrorStatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        )
    })
}

pub fn require_admin(rqctx: &RequestContext<AppState>) -> Result<AuthenticatedUser, HttpError> {
    let user = require_auth(rqctx)?;

    if user.role != "admin" {
        return Err(HttpError::for_client_error(
            None,
            ClientErrorStatusCode::FORBIDDEN,
            "Admin access required".to_string(),
        ));
    }

    Ok(user)
}
