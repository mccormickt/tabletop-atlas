use dropshot::{
    HttpError, HttpResponseCreated, HttpResponseDeleted, HttpResponseHeaders, HttpResponseOk,
};
use schemars::JsonSchema;
use serde::Serialize;

pub mod chat;
pub mod games;
pub mod house_rules;
pub mod upload;

// Re-exports are available but not used globally to avoid namespace pollution

type HttpOk<T> = HttpResponseHeaders<HttpResponseOk<T>, CorsHeaders>;
type HttpCreated<T> = HttpResponseHeaders<HttpResponseCreated<T>, CorsHeaders>;
type HttpDeleted = HttpResponseHeaders<HttpResponseDeleted, CorsHeaders>;

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
