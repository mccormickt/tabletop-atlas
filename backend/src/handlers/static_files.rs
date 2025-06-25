use dropshot::{Body, HttpError, Path as DropPath, RequestContext, endpoint};
use http::{Response, StatusCode};
use include_dir::{Dir, include_dir};
use mime_guess;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AppState,
    handlers::{HttpOk, success_response},
};

// Include the frontend build directory at compile time
static FRONTEND_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../frontend/build");

#[derive(Deserialize, JsonSchema)]
pub struct AssetPathParam {
    pub path: Vec<String>,
}

/// Health check endpoint
#[endpoint {
    method = GET,
    path = "/health",
}]
pub async fn health_check(
    _rqctx: RequestContext<AppState>,
) -> Result<HttpOk<serde_json::Value>, HttpError> {
    let runtime = FRONTEND_ASSETS
        .get_file("index.html")
        .map(|_| "embedded-frontend")
        .unwrap_or("api-only");
    success_response(serde_json::json!({
        "status": "healthy",
        "service": "tabletop-atlas-backend",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "runtime": runtime,
    }))
}

/// Serve favicon
#[endpoint {
    method = GET,
    path = "/favicon.png",
    unpublished = true,
}]
pub async fn serve_favicon(_rqctx: RequestContext<AppState>) -> Result<Response<Body>, HttpError> {
    serve_static_file("favicon.png").await
}

/// Serve SvelteKit app assets
#[endpoint {
    method = GET,
    path = "/_app/{path:.*}",
    unpublished = true,
}]
pub async fn serve_app_assets(
    _rqctx: RequestContext<AppState>,
    path_param: DropPath<AssetPathParam>,
) -> Result<Response<Body>, HttpError> {
    let path_segments = path_param.into_inner().path;
    let asset_path = format!("_app/{}", path_segments.join("/"));
    serve_static_file(&asset_path).await
}

/// Serve the main index.html for the SPA root
#[endpoint {
    method = GET,
    path = "/",
    unpublished = true,
}]
pub async fn serve_index(_rqctx: RequestContext<AppState>) -> Result<Response<Body>, HttpError> {
    serve_spa_fallback().await
}

/// Serve SPA for any games section route
#[endpoint {
    method = GET,
    path = "/games/{path:.*}",
    unpublished = true,
}]
pub async fn serve_games_views(
    _rqctx: RequestContext<AppState>,
    _path_param: DropPath<AssetPathParam>,
) -> Result<Response<Body>, HttpError> {
    serve_spa_fallback().await
}

/// Serve SPA for search route
#[endpoint {
    method = GET,
    path = "/search/{path:.*}",
    unpublished = true,
}]
pub async fn serve_search_view(
    _rqctx: RequestContext<AppState>,
    _path_param: DropPath<AssetPathParam>,
) -> Result<Response<Body>, HttpError> {
    serve_spa_fallback().await
}

/// Serve SPA for upload route
#[endpoint {
    method = GET,
    path = "/upload/{path:.*}",
    unpublished = true,
}]
pub async fn serve_upload_view(
    _rqctx: RequestContext<AppState>,
    _path_param: DropPath<AssetPathParam>,
) -> Result<Response<Body>, HttpError> {
    serve_spa_fallback().await
}

/// Serve SPA for chat route
#[endpoint {
    method = GET,
    path = "/chat/{path:.*}",
    unpublished = true,
}]
pub async fn serve_chat_view(
    _rqctx: RequestContext<AppState>,
    _path_param: DropPath<AssetPathParam>,
) -> Result<Response<Body>, HttpError> {
    serve_spa_fallback().await
}

async fn serve_static_file(path: &str) -> Result<Response<Body>, HttpError> {
    match FRONTEND_ASSETS.get_file(path) {
        Some(file) => serve_static_file_content(path, file.contents()).await,
        None => serve_404().await,
    }
}

async fn serve_static_file_content(
    path: &str,
    content: &[u8],
) -> Result<Response<Body>, HttpError> {
    let content_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Cache-Control", get_cache_control(path))
        .body(Body::from(content.to_vec()))
        .map_err(|e| HttpError::for_internal_error(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

async fn serve_spa_fallback() -> Result<Response<Body>, HttpError> {
    match FRONTEND_ASSETS.get_file("index.html") {
        Some(file) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Cache-Control", "public, max-age=300") // 5 minutes
                .body(Body::from(file.contents().to_vec()))
                .map_err(|e| {
                    HttpError::for_internal_error(format!("Failed to build SPA response: {}", e))
                })?;

            Ok(response)
        }
        None => Err(HttpError::for_internal_error(
            "Frontend index.html not found in embedded assets".to_string(),
        )),
    }
}

async fn serve_404() -> Result<Response<Body>, HttpError> {
    let not_found_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>404 - Not Found</title>
        <style>
            body {
                font-family: sans-serif;
                text-align: center;
                margin-top: 100px;
                background-color: #f8f9fa;
            }
            .error-container {
                background: white;
                padding: 40px;
                border-radius: 8px;
                box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                max-width: 500px;
                margin: 0 auto;
            }
            h1 { color: #e74c3c; margin-bottom: 20px; }
            p { color: #666; margin-bottom: 20px; }
            a { color: #3498db; text-decoration: none; }
            a:hover { text-decoration: underline; }
        </style>
    </head>
    <body>
        <div class="error-container">
            <h1>404 - Not Found</h1>
            <p>The requested resource could not be found.</p>
            <p><a href="/">← Back to Tabletop Atlas</a></p>
            <p><small>Tip: Try navigating to this page from the <a href="/games">Games</a> section.</small></p>
        </div>
    </body>
    </html>
    "#;

    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .body(Body::from(not_found_html.to_string()))
        .map_err(|e| {
            HttpError::for_internal_error(format!("Failed to build 404 response: {}", e))
        })?;

    Ok(response)
}

fn get_cache_control(file_path: &str) -> &'static str {
    if let Some(extension) = std::path::Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())
    {
        match extension {
            // Long cache for assets with hashes in filename
            "js" | "css" | "woff" | "woff2" | "ttf" | "eot" => {
                if file_path.contains("immutable") || file_path.contains("_app/") {
                    "public, max-age=31536000, immutable" // 1 year for hashed assets
                } else {
                    "public, max-age=3600" // 1 hour for other assets
                }
            }
            // Medium cache for images
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "webp" => {
                "public, max-age=86400" // 1 day
            }
            // Short cache for HTML files (for SPA routing)
            "html" => "public, max-age=300", // 5 minutes
            // No cache for unknown types
            _ => "no-cache",
        }
    } else {
        // No extension, likely an HTML route
        "public, max-age=300" // 5 minutes
    }
}
