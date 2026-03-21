use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

use rig::embeddings::EmbeddingModel;

use crate::{
    AppState,
    auth::middleware::require_auth,
    db::{embeddings, house_rules},
    embeddings::Embedder,
    handlers::{
        HttpCreated, HttpDeleted, HttpError, HttpOk, bad_request_error, created_response,
        deleted_response, internal_error, not_found_error, success_response,
    },
    models::{
        CreateEmbeddingRequest, CreateHouseRuleRequest, EmbeddingSourceType, GameId, HouseRule,
        HouseRuleId, PaginatedResponse, UpdateHouseRuleRequest, default_limit, default_page,
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
        .map_err(|e| {
            slog::error!(log, "Failed to delete existing embeddings for house rule";
                "house_rule_id" => house_rule.id, "error" => %e);
            internal_error("Failed to update house rule embeddings".to_string())
        })?;

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
        .map_err(|e| {
            slog::error!(log, "Failed to store embedding for house rule";
                "house_rule_id" => house_rule.id, "error" => %e);
            internal_error("Failed to store house rule embedding".to_string())
        })?;

    slog::info!(log, "Successfully embedded house rule";
        "house_rule_id" => house_rule.id, "game_id" => house_rule.game_id);

    Ok(())
}

#[derive(Deserialize, JsonSchema)]
pub struct HouseRulePathParam {
    pub id: HouseRuleId,
}

#[derive(Deserialize, JsonSchema)]
pub struct HouseRulesByGameQuery {
    pub game_id: GameId,
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

    match house_rules::list_house_rules(&db, query.game_id, query.page, query.limit).await {
        Ok(result) => success_response(result),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to list house rules"; "error" => %e);
            Err(internal_error("Failed to list house rules".to_string()))
        }
    }
}

/// Get a specific house rule by ID
#[endpoint {
    method = GET,
    path = "/api/house-rules/{id}"
}]
pub async fn get_house_rule(
    rqctx: RequestContext<AppState>,
    path: Path<HouseRulePathParam>,
) -> Result<HttpOk<HouseRule>, HttpError> {
    let app_state = rqctx.context();
    let house_rule_id = path.into_inner().id;
    let db = app_state.db();

    match house_rules::get_house_rule(&db, house_rule_id).await {
        Ok(Some(house_rule)) => success_response(house_rule),
        Ok(None) => Err(not_found_error(format!(
            "House rule with id {} not found",
            house_rule_id
        ))),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to get house rule"; "house_rule_id" => house_rule_id, "error" => %e);
            Err(internal_error("Failed to get house rule".to_string()))
        }
    }
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

    match house_rules::create_house_rule(&db, create_request).await {
        Ok(house_rule) => {
            // Embed the house rule asynchronously (don't block on embedding errors)
            if let Err(e) = embed_house_rule(&rqctx.log, embedder, &db, &house_rule).await {
                slog::warn!(rqctx.log, "Failed to embed house rule, continuing";
                    "house_rule_id" => house_rule.id, "error" => ?e);
            }
            created_response(house_rule)
        }
        Err(e) => {
            slog::error!(rqctx.log, "Failed to create house rule"; "error" => %e);
            Err(internal_error("Failed to create house rule".to_string()))
        }
    }
}

/// Update an existing house rule
#[endpoint {
    method = PATCH,
    path = "/api/house-rules/{id}"
}]
pub async fn update_house_rule(
    rqctx: RequestContext<AppState>,
    path: Path<HouseRulePathParam>,
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

    match house_rules::update_house_rule(&db, house_rule_id, update_request).await {
        Ok(Some(house_rule)) => {
            // Re-embed the house rule (handles active state changes)
            if let Err(e) = embed_house_rule(&rqctx.log, embedder, &db, &house_rule).await {
                slog::warn!(rqctx.log, "Failed to re-embed house rule, continuing";
                    "house_rule_id" => house_rule.id, "error" => ?e);
            }
            success_response(house_rule)
        }
        Ok(None) => Err(not_found_error(format!(
            "House rule with id {} not found",
            house_rule_id
        ))),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to update house rule"; "house_rule_id" => house_rule_id, "error" => %e);
            Err(internal_error("Failed to update house rule".to_string()))
        }
    }
}

/// Delete a house rule
#[endpoint {
    method = DELETE,
    path = "/api/house-rules/{id}"
}]
pub async fn delete_house_rule(
    rqctx: RequestContext<AppState>,
    path: Path<HouseRulePathParam>,
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

    match house_rules::delete_house_rule(&db, house_rule_id).await {
        Ok(true) => deleted_response(),
        Ok(false) => Err(not_found_error(format!(
            "House rule with id {} not found",
            house_rule_id
        ))),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to delete house rule"; "house_rule_id" => house_rule_id, "error" => %e);
            Err(internal_error("Failed to delete house rule".to_string()))
        }
    }
}
