use crate::{
    AppState,
    auth::middleware::require_admin,
    db::games,
    error::{DbResultExt, OptionExt},
    handlers::{
        HttpCreated, HttpDeleted, HttpError, HttpOk, bad_request_error, created_response,
        deleted_response, success_response,
    },
    models::{
        CreateGameRequest, Game, GameSummary, PaginatedResponse, UpdateGameRequest, default_limit,
        default_page,
    },
};
use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

use super::IdPath;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GameSearchParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Search query to filter games by name
    pub search: Option<String>,
    /// Filter to only include games that have rules PDFs uploaded
    pub has_rules_pdf: Option<bool>,
}

/// List all games with pagination and optional search
#[endpoint {
    method = GET,
    path = "/api/games"
}]
pub async fn list_games(
    rqctx: RequestContext<AppState>,
    query: Query<GameSearchParams>,
) -> Result<HttpOk<PaginatedResponse<GameSummary>>, HttpError> {
    let app_state = rqctx.context();
    let params = query.into_inner();
    let db = app_state.db();

    let result = games::list_games(
        &db,
        params.page,
        params.limit,
        params.search.as_deref(),
        params.has_rules_pdf,
    )
    .await
    .db_context("Failed to list games")?;

    success_response(result)
}

/// Get a specific game by ID
#[endpoint {
    method = GET,
    path = "/api/games/{id}"
}]
pub async fn get_game(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
) -> Result<HttpOk<Game>, HttpError> {
    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let db = app_state.db();

    let game = games::get_game(&db, game_id)
        .await
        .db_context("Failed to get game")?
        .or_not_found(format!("Game with id {} not found", game_id))?;

    success_response(game)
}

/// Create a new game (admin only)
#[endpoint {
    method = POST,
    path = "/api/games"
}]
pub async fn create_game(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateGameRequest>,
) -> Result<HttpCreated<Game>, HttpError> {
    // Require admin access for creating master games
    require_admin(&rqctx)?;

    let app_state = rqctx.context();
    let create_request = body.into_inner();
    let db = app_state.db();

    // Validate the request
    if create_request.name.trim().is_empty() {
        return Err(bad_request_error("Game name cannot be empty".to_string()));
    }

    if let Some(complexity) = create_request.complexity_rating
        && !(1.0..=5.0).contains(&complexity)
    {
        return Err(bad_request_error(
            "Complexity rating must be between 1.0 and 5.0".to_string(),
        ));
    }

    let game = games::create_game(&db, create_request)
        .await
        .db_context("Failed to create game")?;

    created_response(game)
}

/// Update an existing game (admin only)
#[endpoint {
    method = PATCH,
    path = "/api/games/{id}"
}]
pub async fn update_game(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
    body: TypedBody<UpdateGameRequest>,
) -> Result<HttpOk<Game>, HttpError> {
    // Require admin access for updating master games
    require_admin(&rqctx)?;

    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let update_request = body.into_inner();
    let db = app_state.db();

    // Validate the request
    if let Some(ref name) = update_request.name
        && name.trim().is_empty()
    {
        return Err(bad_request_error("Game name cannot be empty".to_string()));
    }

    if let Some(complexity) = update_request.complexity_rating
        && !(1.0..=5.0).contains(&complexity)
    {
        return Err(bad_request_error(
            "Complexity rating must be between 1.0 and 5.0".to_string(),
        ));
    }

    let game = games::update_game(&db, game_id, update_request)
        .await
        .db_context("Failed to update game")?
        .or_not_found(format!("Game with id {} not found", game_id))?;

    success_response(game)
}

/// Delete a game (admin only)
#[endpoint {
    method = DELETE,
    path = "/api/games/{id}"
}]
pub async fn delete_game(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
) -> Result<HttpDeleted, HttpError> {
    // Require admin access for deleting master games
    require_admin(&rqctx)?;

    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let db = app_state.db();

    let deleted = games::delete_game(&db, game_id)
        .await
        .db_context("Failed to delete game")?;

    if deleted {
        deleted_response()
    } else {
        Err(crate::handlers::not_found_error(format!(
            "Game with id {} not found",
            game_id
        )))
    }
}
