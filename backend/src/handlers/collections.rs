use crate::AppState;
use crate::auth::require_auth;
use crate::db::collections;
use crate::error::{DbResultExt, OptionExt};
use crate::models::{
    AddToCollectionRequest, CollectionEntry, CollectionEntryWithGame, PaginatedResponse,
    PaginationParams, UpdateCollectionRequest,
};
use dropshot::{HttpError, Path, Query, RequestContext, TypedBody, endpoint};

use super::{
    HttpCreated, HttpDeleted, HttpOk, IdPath, created_response, deleted_response, not_found_error,
    success_response,
};

/// List current user's game collection
#[endpoint {
    method = GET,
    path = "/api/collection",
    tags = ["collection"]
}]
pub async fn list_collection(
    rqctx: RequestContext<AppState>,
    query: Query<PaginationParams>,
) -> Result<HttpOk<PaginatedResponse<CollectionEntryWithGame>>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let query = query.into_inner();

    let result = collections::list_user_collection(&db, user.user_id, query.page, query.limit)
        .await
        .db_context("Failed to list collection")?;

    success_response(result)
}

/// Add a game to current user's collection
#[endpoint {
    method = POST,
    path = "/api/collection",
    tags = ["collection"]
}]
pub async fn add_to_collection(
    rqctx: RequestContext<AppState>,
    body: TypedBody<AddToCollectionRequest>,
) -> Result<HttpCreated<CollectionEntry>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let request = body.into_inner();

    let entry = collections::add_to_collection(&db, user.user_id, request)
        .await
        .db_context("Failed to add to collection")?;

    created_response(entry)
}

/// Update a collection entry
#[endpoint {
    method = PATCH,
    path = "/api/collection/{id}",
    tags = ["collection"]
}]
pub async fn update_collection_entry(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
    body: TypedBody<UpdateCollectionRequest>,
) -> Result<HttpOk<CollectionEntry>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let entry_id = path.into_inner().id;
    let request = body.into_inner();

    let entry = collections::update_collection_entry(&db, user.user_id, entry_id, request)
        .await
        .db_context("Failed to update collection entry")?
        .or_not_found("Collection entry not found")?;

    success_response(entry)
}

/// Remove a game from collection
#[endpoint {
    method = DELETE,
    path = "/api/collection/{id}",
    tags = ["collection"]
}]
pub async fn remove_from_collection(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
) -> Result<HttpDeleted, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let entry_id = path.into_inner().id;

    let deleted = collections::remove_from_collection(&db, user.user_id, entry_id)
        .await
        .db_context("Failed to remove from collection")?;

    if !deleted {
        return Err(not_found_error("Collection entry not found".to_string()));
    }

    deleted_response()
}
