use chrono::Utc;
use dropshot::{HttpError, HttpResponseOk, RequestContext, TypedBody, endpoint};
use uuid::Uuid;

use crate::models::{
    AppState, CreateGameRequest, CreateHouseRuleRequest, CreatePdfDocumentRequest, Game,
    GamePathParams, HouseRule, PdfDocument,
};

/// Create a new game
#[endpoint {
    method = POST,
    path = "/games",
}]
pub async fn create_game(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateGameRequest>,
) -> Result<HttpResponseOk<Game>, HttpError> {
    let state = rqctx.context();
    let request = body.into_inner();

    let game = Game {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        created_at: Utc::now().to_rfc3339(),
    };

    state.add_game(game.clone());

    Ok(HttpResponseOk(game))
}

/// Get all games
#[endpoint {
    method = GET,
    path = "/games",
}]
pub async fn list_games(
    rqctx: RequestContext<AppState>,
) -> Result<HttpResponseOk<Vec<Game>>, HttpError> {
    let state = rqctx.context();
    let games = state.get_all_games();

    Ok(HttpResponseOk(games))
}

/// Get a specific game by ID
#[endpoint {
    method = GET,
    path = "/games/{game_id}",
}]
pub async fn get_game(
    rqctx: RequestContext<AppState>,
    path_params: dropshot::Path<GamePathParams>,
) -> Result<HttpResponseOk<Game>, HttpError> {
    let state = rqctx.context();
    let game_id = path_params.into_inner().game_id;

    match state.get_game(&game_id) {
        Some(game) => Ok(HttpResponseOk(game)),
        None => Err(HttpError::for_not_found(None, "Game not found".to_string())),
    }
}

/// Upload a PDF document for a game
#[endpoint {
    method = POST,
    path = "/games/{game_id}/documents",
}]
pub async fn upload_pdf_document(
    rqctx: RequestContext<AppState>,
    path_params: dropshot::Path<GamePathParams>,
    body: TypedBody<CreatePdfDocumentRequest>,
) -> Result<HttpResponseOk<PdfDocument>, HttpError> {
    let state = rqctx.context();
    let game_id = path_params.into_inner().game_id;
    let request = body.into_inner();

    // Verify game exists
    if !state.game_exists(&game_id) {
        return Err(HttpError::for_not_found(None, "Game not found".to_string()));
    }

    // Verify game_id matches path parameter
    if request.game_id != game_id {
        return Err(HttpError::for_bad_request(
            None,
            "Game ID in path and body must match".to_string(),
        ));
    }

    let pdf_document = PdfDocument {
        id: Uuid::new_v4(),
        game_id: request.game_id,
        filename: request.filename,
        content: request.content,
        file_size: request.file_size,
        uploaded_at: Utc::now().to_rfc3339(),
        processed: false,
        chunk_count: None,
    };

    state.add_pdf_document(pdf_document.clone());

    Ok(HttpResponseOk(pdf_document))
}

/// Get all PDF documents for a specific game
#[endpoint {
    method = GET,
    path = "/games/{game_id}/documents",
}]
pub async fn list_pdf_documents_for_game(
    rqctx: RequestContext<AppState>,
    path_params: dropshot::Path<GamePathParams>,
) -> Result<HttpResponseOk<Vec<PdfDocument>>, HttpError> {
    let state = rqctx.context();
    let game_id = path_params.into_inner().game_id;

    // Verify game exists
    if !state.game_exists(&game_id) {
        return Err(HttpError::for_not_found(None, "Game not found".to_string()));
    }

    let pdf_documents = state.get_pdf_documents_for_game(&game_id);

    Ok(HttpResponseOk(pdf_documents))
}

/// Create a new house rule for a game
#[endpoint {
    method = POST,
    path = "/house-rules",
}]
pub async fn create_house_rule(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateHouseRuleRequest>,
) -> Result<HttpResponseOk<HouseRule>, HttpError> {
    let state = rqctx.context();
    let request = body.into_inner();

    // Verify game exists
    if !state.game_exists(&request.game_id) {
        return Err(HttpError::for_not_found(None, "Game not found".to_string()));
    }

    let house_rule = HouseRule {
        id: Uuid::new_v4(),
        game_id: request.game_id,
        title: request.title,
        content: request.content,
        created_by: request.created_by,
        created_at: Utc::now().to_rfc3339(),
    };

    state.add_house_rule(house_rule.clone());

    Ok(HttpResponseOk(house_rule))
}

/// Get all house rules for a specific game
#[endpoint {
    method = GET,
    path = "/games/{game_id}/house-rules",
}]
pub async fn list_house_rules_for_game(
    rqctx: RequestContext<AppState>,
    path_params: dropshot::Path<GamePathParams>,
) -> Result<HttpResponseOk<Vec<HouseRule>>, HttpError> {
    let state = rqctx.context();
    let game_id = path_params.into_inner().game_id;

    // Verify game exists
    if !state.game_exists(&game_id) {
        return Err(HttpError::for_not_found(None, "Game not found".to_string()));
    }

    let house_rules = state.get_house_rules_for_game(&game_id);

    Ok(HttpResponseOk(house_rules))
}
