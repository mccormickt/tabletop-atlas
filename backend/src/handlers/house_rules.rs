use dropshot::{HttpError, Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AppState,
    db::{Database, house_rules},
    handlers::{
        HttpCreated, HttpDeleted, HttpOk, created_response, deleted_response, success_response,
    },
    models::{
        CreateHouseRuleRequest, GameId, HouseRule, HouseRuleId, PaginatedResponse,
        PaginationParams, UpdateHouseRuleRequest,
    },
};

#[derive(Deserialize, JsonSchema)]
pub struct HouseRulePathParam {
    pub id: HouseRuleId,
}

#[derive(Deserialize, JsonSchema)]
pub struct HouseRulesByGameQuery {
    pub game_id: GameId,
    #[serde(flatten)]
    pub pagination: PaginationParams,
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
    let db = Database::new(app_state.db());

    match house_rules::list_house_rules(
        &db,
        query.game_id,
        query.pagination.page,
        query.pagination.limit,
    )
    .await
    {
        Ok(result) => success_response(result),
        Err(e) => {
            tracing::error!("Failed to list house rules: {}", e);
            Err(HttpError::for_internal_error(
                "Failed to list house rules".to_string(),
            ))
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
    let db = Database::new(app_state.db());

    match house_rules::get_house_rule(&db, house_rule_id).await {
        Ok(Some(house_rule)) => success_response(house_rule),
        Ok(None) => Err(HttpError::for_not_found(
            None,
            format!("House rule with id {} not found", house_rule_id),
        )),
        Err(e) => {
            tracing::error!("Failed to get house rule {}: {}", house_rule_id, e);
            Err(HttpError::for_internal_error(
                "Failed to get house rule".to_string(),
            ))
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
    let app_state = rqctx.context();
    let create_request = body.into_inner();
    let db = Database::new(app_state.db());

    // Validate the request
    if create_request.title.trim().is_empty() {
        return Err(HttpError::for_bad_request(
            None,
            "House rule title cannot be empty".to_string(),
        ));
    }
    if create_request.description.trim().is_empty() {
        return Err(HttpError::for_bad_request(
            None,
            "House rule description cannot be empty".to_string(),
        ));
    }

    match house_rules::create_house_rule(&db, create_request).await {
        Ok(house_rule) => created_response(house_rule),
        Err(e) => {
            tracing::error!("Failed to create house rule: {}", e);
            Err(HttpError::for_internal_error(
                "Failed to create house rule".to_string(),
            ))
        }
    }
}

/// Update an existing house rule
#[endpoint {
    method = PUT,
    path = "/api/house-rules/{id}"
}]
pub async fn update_house_rule(
    rqctx: RequestContext<AppState>,
    path: Path<HouseRulePathParam>,
    body: TypedBody<UpdateHouseRuleRequest>,
) -> Result<HttpOk<HouseRule>, HttpError> {
    let app_state = rqctx.context();
    let house_rule_id = path.into_inner().id;
    let update_request = body.into_inner();
    let db = Database::new(app_state.db());

    // Validate the request
    if let Some(ref title) = update_request.title {
        if title.trim().is_empty() {
            return Err(HttpError::for_bad_request(
                None,
                "House rule title cannot be empty".to_string(),
            ));
        }
    }
    if let Some(ref description) = update_request.description {
        if description.trim().is_empty() {
            return Err(HttpError::for_bad_request(
                None,
                "House rule description cannot be empty".to_string(),
            ));
        }
    }

    match house_rules::update_house_rule(&db, house_rule_id, update_request).await {
        Ok(Some(house_rule)) => success_response(house_rule),
        Ok(None) => Err(HttpError::for_not_found(
            None,
            format!("House rule with id {} not found", house_rule_id),
        )),
        Err(e) => {
            tracing::error!("Failed to update house rule {}: {}", house_rule_id, e);
            Err(HttpError::for_internal_error(
                "Failed to update house rule".to_string(),
            ))
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
    let app_state = rqctx.context();
    let house_rule_id = path.into_inner().id;
    let db = Database::new(app_state.db());

    match house_rules::delete_house_rule(&db, house_rule_id).await {
        Ok(true) => deleted_response(),
        Ok(false) => Err(HttpError::for_not_found(
            None,
            format!("House rule with id {} not found", house_rule_id),
        )),
        Err(e) => {
            tracing::error!("Failed to delete house rule {}: {}", house_rule_id, e);
            Err(HttpError::for_internal_error(
                "Failed to delete house rule".to_string(),
            ))
        }
    }
}
