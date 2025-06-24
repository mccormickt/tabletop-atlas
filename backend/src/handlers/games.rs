use crate::{
    AppState,
    db::{Database, games},
    handlers::{
        HttpCreated, HttpDeleted, HttpError, HttpOk, bad_request_error, created_response,
        deleted_response, internal_error, not_found_error, success_response,
    },
    models::{
        CreateGameRequest, Game, GameId, GameSummary, PaginatedResponse, PaginationParams,
        UpdateGameRequest,
    },
};
use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct GamePathParam {
    pub id: GameId,
}

/// List all games with pagination
#[endpoint {
    method = GET,
    path = "/api/games"
}]
pub async fn list_games(
    rqctx: RequestContext<AppState>,
    query: Query<PaginationParams>,
) -> Result<HttpOk<PaginatedResponse<GameSummary>>, HttpError> {
    let app_state = rqctx.context();
    let pagination = query.into_inner();
    let db = app_state.db();

    match games::list_games(&db, pagination.page, pagination.limit).await {
        Ok(result) => success_response(result),
        Err(e) => {
            tracing::error!("Failed to list games: {}", e);
            Err(internal_error("Failed to list games".to_string()))
        }
    }
}

/// Get a specific game by ID
#[endpoint {
    method = GET,
    path = "/api/games/{id}"
}]
pub async fn get_game(
    rqctx: RequestContext<AppState>,
    path: Path<GamePathParam>,
) -> Result<HttpOk<Game>, HttpError> {
    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let db = app_state.db();

    match games::get_game(&db, game_id).await {
        Ok(Some(game)) => success_response(game),
        Ok(None) => Err(not_found_error(format!(
            "Game with id {} not found",
            game_id
        ))),
        Err(e) => {
            tracing::error!("Failed to get game {}: {}", game_id, e);
            Err(internal_error("Failed to get game".to_string()))
        }
    }
}

/// Create a new game
#[endpoint {
    method = POST,
    path = "/api/games"
}]
pub async fn create_game(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateGameRequest>,
) -> Result<HttpCreated<Game>, HttpError> {
    let app_state = rqctx.context();
    let create_request = body.into_inner();
    let db = app_state.db();

    // Validate the request
    if create_request.name.trim().is_empty() {
        return Err(bad_request_error("Game name cannot be empty".to_string()));
    }

    if let Some(complexity) = create_request.complexity_rating {
        if complexity < 1.0 || complexity > 5.0 {
            return Err(bad_request_error(
                "Complexity rating must be between 1.0 and 5.0".to_string(),
            ));
        }
    }

    match games::create_game(&db, create_request).await {
        Ok(game) => created_response(game),
        Err(e) => {
            tracing::error!("Failed to create game: {}", e);
            Err(internal_error("Failed to create game".to_string()))
        }
    }
}

/// Update an existing game
#[endpoint {
    method = PUT,
    path = "/api/games/{id}"
}]
pub async fn update_game(
    rqctx: RequestContext<AppState>,
    path: Path<GamePathParam>,
    body: TypedBody<UpdateGameRequest>,
) -> Result<HttpOk<Game>, HttpError> {
    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let update_request = body.into_inner();
    let db = app_state.db();

    // Validate the request
    if let Some(ref name) = update_request.name {
        if name.trim().is_empty() {
            return Err(bad_request_error("Game name cannot be empty".to_string()));
        }
    }

    if let Some(complexity) = update_request.complexity_rating {
        if complexity < 1.0 || complexity > 5.0 {
            return Err(bad_request_error(
                "Complexity rating must be between 1.0 and 5.0".to_string(),
            ));
        }
    }

    match games::update_game(&db, game_id, update_request).await {
        Ok(Some(game)) => success_response(game),
        Ok(None) => Err(not_found_error(format!(
            "Game with id {} not found",
            game_id
        ))),
        Err(e) => {
            tracing::error!("Failed to update game {}: {}", game_id, e);
            Err(internal_error("Failed to update game".to_string()))
        }
    }
}

/// Delete a game
#[endpoint {
    method = DELETE,
    path = "/api/games/{id}"
}]
pub async fn delete_game(
    rqctx: RequestContext<AppState>,
    path: Path<GamePathParam>,
) -> Result<HttpDeleted, HttpError> {
    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let db = app_state.db();

    match games::delete_game(&db, game_id).await {
        Ok(true) => deleted_response(),
        Ok(false) => Err(not_found_error(format!(
            "Game with id {} not found",
            game_id
        ))),
        Err(e) => {
            tracing::error!("Failed to delete game {}: {}", game_id, e);
            Err(internal_error("Failed to delete game".to_string()))
        }
    }
}
