use dropshot::{HttpError, Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::AppState;
use crate::auth::{extract_auth, require_auth};
use crate::db::custom_games;
use crate::models::{
    CreateCustomGameRequest, CustomGame, CustomGameSummary, PaginatedResponse, PaginationParams,
    UpdateCustomGameRequest,
};

use super::{
    HttpCreated, HttpDeleted, HttpOk, created_response, deleted_response, internal_error,
    not_found_error, success_response,
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CustomGamePath {
    pub id: i64,
}

/// List current user's custom games
#[endpoint {
    method = GET,
    path = "/api/custom-games",
    tags = ["custom_games"]
}]
pub async fn list_custom_games(
    rqctx: RequestContext<AppState>,
    query: Query<PaginationParams>,
) -> Result<HttpOk<PaginatedResponse<CustomGameSummary>>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let query = query.into_inner();

    let result = custom_games::list_user_custom_games(&db, user.user_id, query.page, query.limit)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    success_response(result)
}

/// List public custom games (browsable by anyone)
#[endpoint {
    method = GET,
    path = "/api/public-games",
    tags = ["custom_games"]
}]
pub async fn list_public_custom_games(
    rqctx: RequestContext<AppState>,
    query: Query<PaginationParams>,
) -> Result<HttpOk<PaginatedResponse<CustomGameSummary>>, HttpError> {
    let db = rqctx.context().db();
    let query = query.into_inner();

    let result = custom_games::list_public_custom_games(&db, query.page, query.limit)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    success_response(result)
}

/// Create a custom game
#[endpoint {
    method = POST,
    path = "/api/custom-games",
    tags = ["custom_games"]
}]
pub async fn create_custom_game(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateCustomGameRequest>,
) -> Result<HttpCreated<CustomGame>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let request = body.into_inner();

    let game = custom_games::create_custom_game(&db, user.user_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    created_response(game)
}

/// Get a custom game (public games visible to all, private only to owner)
#[endpoint {
    method = GET,
    path = "/api/custom-games/{id}",
    tags = ["custom_games"]
}]
pub async fn get_custom_game(
    rqctx: RequestContext<AppState>,
    path: Path<CustomGamePath>,
) -> Result<HttpOk<CustomGame>, HttpError> {
    let user = extract_auth(&rqctx);
    let db = rqctx.context().db();
    let game_id = path.into_inner().id;

    let game = custom_games::get_custom_game(&db, game_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Custom game not found".to_string()))?;

    // Check access: public games visible to all, private only to owner
    if !game.is_public {
        match user {
            Some(u) if u.user_id == game.user_id => {}
            _ => {
                return Err(HttpError::for_client_error(
                    None,
                    dropshot::ClientErrorStatusCode::FORBIDDEN,
                    "Access denied".to_string(),
                ));
            }
        }
    }

    success_response(game)
}

/// Update a custom game (owner only)
#[endpoint {
    method = PATCH,
    path = "/api/custom-games/{id}",
    tags = ["custom_games"]
}]
pub async fn update_custom_game(
    rqctx: RequestContext<AppState>,
    path: Path<CustomGamePath>,
    body: TypedBody<UpdateCustomGameRequest>,
) -> Result<HttpOk<CustomGame>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let game_id = path.into_inner().id;
    let request = body.into_inner();

    let game = custom_games::update_custom_game(&db, user.user_id, game_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Custom game not found or access denied".to_string()))?;

    success_response(game)
}

/// Delete a custom game (owner only)
#[endpoint {
    method = DELETE,
    path = "/api/custom-games/{id}",
    tags = ["custom_games"]
}]
pub async fn delete_custom_game(
    rqctx: RequestContext<AppState>,
    path: Path<CustomGamePath>,
) -> Result<HttpDeleted, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let game_id = path.into_inner().id;

    let deleted = custom_games::delete_custom_game(&db, user.user_id, game_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !deleted {
        return Err(not_found_error(
            "Custom game not found or access denied".to_string(),
        ));
    }

    deleted_response()
}
