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
    let cookie_header = request.headers().get("cookie")?.to_str().ok()?;
    let cookies = parse_cookies(cookie_header);
    let access_token = cookies.get("access_token")?;

    let claims = jwt::verify_access_token(access_token).ok()?;

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

#[allow(dead_code)]
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
