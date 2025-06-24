use dropshot::{
    HttpError, HttpResponseCreated, HttpResponseDeleted, HttpResponseHeaders, HttpResponseOk,
};
use schemars::JsonSchema;
use serde::Serialize;

pub mod chat;
pub mod games;
pub mod house_rules;
pub mod static_files;
pub mod upload;

// Re-exports are available but not used globally to avoid namespace pollution

type HttpOk<T> = HttpResponseHeaders<HttpResponseOk<T>, CorsHeaders>;
type HttpCreated<T> = HttpResponseHeaders<HttpResponseCreated<T>, CorsHeaders>;
type HttpDeleted = HttpResponseHeaders<HttpResponseDeleted, CorsHeaders>;

/// Helper function for internal server errors
pub fn internal_error(message: String) -> HttpError {
    let cors_headers = default_cors_headers();
    HttpError::for_internal_error(message)
        .with_header("Access-Control-Allow-Origin", &cors_headers.origin)
        .expect("Failed to add CORS headers")
        .with_header("Access-Control-Allow-Methods", &cors_headers.methods)
        .expect("Failed to add CORS headers")
        .with_header("Access-Control-Allow-Headers", &cors_headers.headers)
        .expect("Failed to add CORS headers")
}

/// Helper function for not found errors
pub fn not_found_error(message: String) -> HttpError {
    let cors_headers = default_cors_headers();
    HttpError::for_not_found(None, message)
        .with_header("Access-Control-Allow-Origin", &cors_headers.origin)
        .expect("Failed to add CORS headers")
        .with_header("Access-Control-Allow-Methods", &cors_headers.methods)
        .expect("Failed to add CORS headers")
        .with_header("Access-Control-Allow-Headers", &cors_headers.headers)
        .expect("Failed to add CORS headers")
}

/// Helper function for bad request errors
pub fn bad_request_error(message: String) -> HttpError {
    let cors_headers = default_cors_headers();
    HttpError::for_bad_request(None, message)
        .with_header("Access-Control-Allow-Origin", &cors_headers.origin)
        .expect("Failed to add CORS headers")
        .with_header("Access-Control-Allow-Methods", &cors_headers.methods)
        .expect("Failed to add CORS headers")
        .with_header("Access-Control-Allow-Headers", &cors_headers.headers)
        .expect("Failed to add CORS headers")
}

/// Constant CORS headers configuration
fn default_cors_headers() -> CorsHeaders {
    CorsHeaders {
        origin: String::from("*"),
        methods: String::from("GET, POST, PUT, DELETE, OPTIONS"),
        headers: String::from("Content-Type, Authorization"),
    }
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
