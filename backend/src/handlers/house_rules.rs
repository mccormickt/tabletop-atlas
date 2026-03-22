use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

use rig::embeddings::EmbeddingModel;

use crate::{
    AppState,
    auth::middleware::require_auth,
    db::{embeddings, house_rules},
    embeddings::Embedder,
    error::{DbResultExt, OptionExt},
    handlers::{
        HttpCreated, HttpDeleted, HttpError, HttpOk, IdPath, bad_request_error, created_response,
        deleted_response, internal_error, success_response,
    },
    models::{
        CreateEmbeddingRequest, CreateHouseRuleRequest, EmbeddingSourceType, HouseRule,
        PaginatedResponse, UpdateHouseRuleRequest, default_limit, default_page,
    },
};

/// Embed a house rule into the vector database
async fn embed_house_rule<M: EmbeddingModel>(
    log: &slog::Logger,
    embedder: &Embedder<M>,
    db: &crate::db::Database,
    house_rule: &HouseRule,
) -> Result<(), HttpError> {
    // First, delete any existing embeddings for this house rule
    embeddings::delete_embeddings_for_house_rule(db, house_rule.id)
        .await
        .db_context("Failed to update house rule embeddings")?;

    // If the house rule is not active, we're done (don't embed inactive rules)
    if !house_rule.is_active {
        slog::info!(log, "House rule is inactive, skipping embedding";
            "house_rule_id" => house_rule.id);
        return Ok(());
    }

    // Create the embedding text from the house rule
    let embedding_text = format!(
        "House Rule: {}\n{}\n{}",
        house_rule.title,
        house_rule.description,
        house_rule
            .category
            .as_ref()
            .map(|c| format!("Category: {}", c))
            .unwrap_or_default()
    );

    // Generate the embedding
    let embedding = embedder
        .generate_embedding(&embedding_text)
        .await
        .map_err(|e| {
            slog::error!(log, "Failed to generate embedding for house rule";
                "house_rule_id" => house_rule.id, "error" => %e);
            internal_error("Failed to generate embedding for house rule".to_string())
        })?;

    // Store the embedding
    let create_request = CreateEmbeddingRequest {
        game_id: house_rule.game_id,
        chunk_text: embedding_text,
        embedding,
        chunk_index: 0, // House rules are single chunks
        source_type: EmbeddingSourceType::HouseRule,
        source_id: Some(house_rule.id),
        metadata: Some(
            serde_json::json!({
                "title": house_rule.title,
                "category": house_rule.category,
            })
            .to_string(),
        ),
    };

    embeddings::create_embedding(db, create_request)
        .await
        .db_context("Failed to store house rule embedding")?;

    slog::info!(log, "Successfully embedded house rule";
        "house_rule_id" => house_rule.id, "game_id" => house_rule.game_id);

    Ok(())
}

#[derive(Deserialize, JsonSchema)]
pub struct HouseRulesByGameQuery {
    pub game_id: i64,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// List house rules for a specific game
#[endpoint {
    method = GET,
    path = "/api/house-rules"
}]
pub async fn list_house_rules(
    rqctx: RequestContext<AppState>,
    query: Query<HouseRulesByGameQuery>,
) -> Result<HttpOk<PaginatedResponse<HouseRule>>, HttpError> {
    let app_state = rqctx.context();
    let query = query.into_inner();
    let db = app_state.db();

    let result = house_rules::list_house_rules(&db, query.game_id, query.page, query.limit)
        .await
        .db_context("Failed to list house rules")?;

    success_response(result)
}

/// Get a specific house rule by ID
#[endpoint {
    method = GET,
    path = "/api/house-rules/{id}"
}]
pub async fn get_house_rule(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
) -> Result<HttpOk<HouseRule>, HttpError> {
    let app_state = rqctx.context();
    let house_rule_id = path.into_inner().id;
    let db = app_state.db();

    let house_rule = house_rules::get_house_rule(&db, house_rule_id)
        .await
        .db_context("Failed to get house rule")?
        .or_not_found(format!("House rule with id {} not found", house_rule_id))?;

    success_response(house_rule)
}

/// Create a new house rule
#[endpoint {
    method = POST,
    path = "/api/house-rules"
}]
pub async fn create_house_rule(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateHouseRuleRequest>,
) -> Result<HttpCreated<HouseRule>, HttpError> {
    require_auth(&rqctx)?;

    let app_state = rqctx.context();
    let create_request = body.into_inner();
    let db = app_state.db();
    let embedder = app_state.embedder();

    // Validate the request
    if create_request.title.trim().is_empty() {
        return Err(bad_request_error(
            "House rule title cannot be empty".to_string(),
        ));
    }
    if create_request.description.trim().is_empty() {
        return Err(bad_request_error(
            "House rule description cannot be empty".to_string(),
        ));
    }

    let house_rule = house_rules::create_house_rule(&db, create_request)
        .await
        .db_context("Failed to create house rule")?;

    // Embed the house rule asynchronously (don't block on embedding errors)
    if let Err(e) = embed_house_rule(&rqctx.log, embedder, &db, &house_rule).await {
        slog::warn!(rqctx.log, "Failed to embed house rule, continuing";
            "house_rule_id" => house_rule.id, "error" => ?e);
    }

    created_response(house_rule)
}

/// Update an existing house rule
#[endpoint {
    method = PATCH,
    path = "/api/house-rules/{id}"
}]
pub async fn update_house_rule(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
    body: TypedBody<UpdateHouseRuleRequest>,
) -> Result<HttpOk<HouseRule>, HttpError> {
    require_auth(&rqctx)?;

    let app_state = rqctx.context();
    let house_rule_id = path.into_inner().id;
    let update_request = body.into_inner();
    let db = app_state.db();
    let embedder = app_state.embedder();

    // Validate the request
    if let Some(ref title) = update_request.title
        && title.trim().is_empty()
    {
        return Err(bad_request_error(
            "House rule title cannot be empty".to_string(),
        ));
    }
    if let Some(ref description) = update_request.description
        && description.trim().is_empty()
    {
        return Err(bad_request_error(
            "House rule description cannot be empty".to_string(),
        ));
    }

    let house_rule = house_rules::update_house_rule(&db, house_rule_id, update_request)
        .await
        .db_context("Failed to update house rule")?
        .or_not_found(format!("House rule with id {} not found", house_rule_id))?;

    // Re-embed the house rule (handles active state changes)
    if let Err(e) = embed_house_rule(&rqctx.log, embedder, &db, &house_rule).await {
        slog::warn!(rqctx.log, "Failed to re-embed house rule, continuing";
            "house_rule_id" => house_rule.id, "error" => ?e);
    }

    success_response(house_rule)
}

/// Delete a house rule
#[endpoint {
    method = DELETE,
    path = "/api/house-rules/{id}"
}]
pub async fn delete_house_rule(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
) -> Result<HttpDeleted, HttpError> {
    require_auth(&rqctx)?;

    let app_state = rqctx.context();
    let house_rule_id = path.into_inner().id;
    let db = app_state.db();

    // Delete embeddings for this house rule first
    if let Err(e) = embeddings::delete_embeddings_for_house_rule(&db, house_rule_id).await {
        slog::warn!(rqctx.log, "Failed to delete embeddings for house rule, continuing";
            "house_rule_id" => house_rule_id, "error" => %e);
    }

    let deleted = house_rules::delete_house_rule(&db, house_rule_id)
        .await
        .db_context("Failed to delete house rule")?;

    if deleted {
        deleted_response()
    } else {
        Err(crate::handlers::not_found_error(format!(
            "House rule with id {} not found",
            house_rule_id
        )))
    }
}
