// Dropshot's #[endpoint] macro generates NeedRequestContext structs that trigger dead_code warnings
#![allow(dead_code)]

use dropshot::{
    HttpError, HttpResponseCreated, HttpResponseDeleted, HttpResponseHeaders, HttpResponseOk,
};
use schemars::JsonSchema;
use serde::Serialize;

pub mod admin;
pub mod auth;
pub mod challenges;
pub mod chat;
pub mod collections;
pub mod custom_games;
pub mod games;
pub mod house_rules;
pub mod static_files;
pub mod tools;
pub mod upload;

// Re-exports are available but not used globally to avoid namespace pollution

type HttpOk<T> = HttpResponseHeaders<HttpResponseOk<T>, CorsHeaders>;
type HttpCreated<T> = HttpResponseHeaders<HttpResponseCreated<T>, CorsHeaders>;
type HttpDeleted = HttpResponseHeaders<HttpResponseDeleted, CorsHeaders>;

/// Helper function for internal server errors
pub fn internal_error(message: String) -> HttpError {
    add_cors_headers(HttpError::for_internal_error(message))
}

/// Helper function for not found errors
pub fn not_found_error(message: String) -> HttpError {
    add_cors_headers(HttpError::for_not_found(None, message))
}

/// Helper function for bad request errors
pub fn bad_request_error(message: String) -> HttpError {
    add_cors_headers(HttpError::for_bad_request(None, message))
}

/// Helper function for unauthorized errors
pub fn unauthorized_error(message: String) -> HttpError {
    add_cors_headers(HttpError::for_client_error(
        None,
        dropshot::ClientErrorStatusCode::UNAUTHORIZED,
        message,
    ))
}

/// Helper function for forbidden errors
pub fn forbidden_error(message: String) -> HttpError {
    add_cors_headers(HttpError::for_client_error(
        None,
        dropshot::ClientErrorStatusCode::FORBIDDEN,
        message,
    ))
}

/// Constant CORS headers configuration
fn default_cors_headers() -> CorsHeaders {
    CorsHeaders {
        origin: String::from("*"),
        methods: String::from("GET, POST, PUT, PATCH, DELETE, OPTIONS"),
        headers: String::from("Content-Type, Authorization"),
    }
}

/// Helper to add CORS headers to an HttpError
fn add_cors_headers(error: HttpError) -> HttpError {
    let cors_headers = default_cors_headers();
    error
        .with_header("Access-Control-Allow-Origin", &cors_headers.origin)
        .and_then(|e| e.with_header("Access-Control-Allow-Methods", &cors_headers.methods))
        .and_then(|e| e.with_header("Access-Control-Allow-Headers", &cors_headers.headers))
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to add CORS headers: {:?}", e);
            e.into()
        })
}

#[derive(Serialize, JsonSchema)]
pub struct CorsHeaders {
    #[serde(rename = "Access-Control-Allow-Origin")]
    pub origin: String,
    #[serde(rename = "Access-Control-Allow-Methods")]
    pub methods: String,
    #[serde(rename = "Access-Control-Allow-Headers")]
    pub headers: String,
}

/// Common response helper with CORS headers
pub fn success_response<T>(data: T) -> Result<HttpOk<T>, HttpError>
where
    T: Serialize + JsonSchema + Send + Sync + 'static,
{
    let headers = default_cors_headers();
    Ok(HttpResponseHeaders::new(HttpResponseOk(data), headers))
}

/// Common response helper with CORS headers
pub fn created_response<T>(data: T) -> Result<HttpCreated<T>, HttpError>
where
    T: Serialize + JsonSchema + Send + Sync + 'static,
{
    let headers = default_cors_headers();
    Ok(HttpResponseHeaders::new(HttpResponseCreated(data), headers))
}

/// Common response helper with CORS headers
pub fn deleted_response() -> Result<HttpDeleted, HttpError> {
    let headers = default_cors_headers();
    Ok(HttpResponseHeaders::new(HttpResponseDeleted(), headers))
}
