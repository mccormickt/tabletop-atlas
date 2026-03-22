use crate::{
    AppState,
    auth::middleware::require_admin,
    bgg::BggClient,
    db::admin as admin_db,
    db::users,
    handlers::{bad_request_error, forbidden_error, success_response},
    models::Game,
    models::admin::{
        BggEnrichError, BggEnrichPreviewResponse, BggEnrichRequest, BggGameEnrichPreview,
        BggGamePreview, BggGameUpdatePreview, BggGameValues, BggImportPreviewResponse,
        BggImportResponse, BggParseError, BulkEnrichPreviewResponse, BulkEnrichRequest,
        BulkEnrichResponse, EnrichmentStats, FieldChange, ParsedBggGame,
    },
    models::{PaginatedResponse, UpdateUserRoleRequest, UserListItem, default_limit, default_page},
};
use dropshot::{Path, Query, RequestContext, TypedBody, UntypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{HttpError, HttpOk, IdPath};
use crate::error::{DbResultExt, OptionExt};

// Maximum file size: 15MB
const MAX_CSV_SIZE: usize = 15 * 1024 * 1024;

/// Admin dashboard stats
#[derive(Debug, Serialize, JsonSchema)]
pub struct AdminDashboardStats {
    pub master_games_count: u32,
}

/// Get admin dashboard stats
#[endpoint {
    method = GET,
    path = "/api/admin/stats"
}]
pub async fn get_admin_stats(
    rqctx: RequestContext<AppState>,
) -> Result<HttpOk<AdminDashboardStats>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let app_state = rqctx.context();
    let db = app_state.db();

    let master_games_count = admin_db::get_master_games_count(&db)
        .await
        .db_context("Failed to get stats")?;

    success_response(AdminDashboardStats { master_games_count })
}

// ============================================================================
// User Management Endpoints
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UserSearchParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub search: Option<String>,
    pub role: Option<String>,
}

/// List users with pagination, search, and role filter (admin only)
#[endpoint {
    method = GET,
    path = "/api/admin/users"
}]
pub async fn list_admin_users(
    rqctx: RequestContext<AppState>,
    query: Query<UserSearchParams>,
) -> Result<HttpOk<PaginatedResponse<UserListItem>>, HttpError> {
    require_admin(&rqctx)?;

    let params = query.into_inner();
    let app_state = rqctx.context();
    let db = app_state.db();

    // Validate role filter if provided
    if let Some(ref role) = params.role
        && role != "admin"
        && role != "user"
    {
        return Err(bad_request_error(format!(
            "Invalid role filter: '{}'. Must be 'admin' or 'user'",
            role
        )));
    }

    let limit = params.limit.min(100);
    let result = users::list_users(
        &db,
        params.page,
        limit,
        params.search.as_deref(),
        params.role.as_deref(),
    )
    .await
    .db_context("Failed to list users")?;

    success_response(result)
}

/// Update a user's role (admin only)
#[endpoint {
    method = PUT,
    path = "/api/admin/users/{id}/role"
}]
pub async fn update_user_role(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
    body: TypedBody<UpdateUserRoleRequest>,
) -> Result<HttpOk<UserListItem>, HttpError> {
    let admin = require_admin(&rqctx)?;

    let target_user_id = path.into_inner().id;
    let request = body.into_inner();
    let app_state = rqctx.context();
    let db = app_state.db();

    // Validate role value
    if request.role != "admin" && request.role != "user" {
        return Err(bad_request_error(format!(
            "Invalid role: '{}'. Must be 'admin' or 'user'",
            request.role
        )));
    }

    // Cannot change own role
    if admin.user_id == target_user_id {
        return Err(forbidden_error("Cannot change your own role".to_string()));
    }

    // The last-admin check is done atomically inside the DB transaction.
    // Returns AppError::BadRequest if demoting last admin, AppError::Db for DB errors.
    let check_last_admin = request.role == "user";
    let updated = users::update_user_role(&db, target_user_id, &request.role, check_last_admin)
        .await?
        .or_not_found(format!("User {} not found", target_user_id))?;

    success_response(updated)
}

// ============================================================================
// BGG Import Endpoints
// ============================================================================

/// Preview BGG CSV import (shows what will be inserted/updated)
#[endpoint {
    method = POST,
    path = "/api/admin/games/import/preview"
}]
pub async fn preview_bgg_import(
    rqctx: RequestContext<AppState>,
    body: UntypedBody,
) -> Result<HttpOk<BggImportPreviewResponse>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let body_bytes = body.as_bytes();

    // Validate file size
    if body_bytes.is_empty() {
        return Err(bad_request_error("No file data provided".to_string()));
    }
    if body_bytes.len() > MAX_CSV_SIZE {
        return Err(bad_request_error(format!(
            "File too large. Maximum size is {} MB",
            MAX_CSV_SIZE / 1024 / 1024
        )));
    }

    // Parse CSV
    let (parsed_games, errors) = parse_bgg_csv(body_bytes)?;

    if parsed_games.is_empty() && !errors.is_empty() {
        return Err(bad_request_error(format!(
            "Failed to parse CSV: {}",
            errors
                .first()
                .map(|e| e.message.as_str())
                .unwrap_or("Unknown error")
        )));
    }

    // Get existing games by bgg_id to determine inserts vs updates
    let app_state = rqctx.context();
    let db = app_state.db();

    let bgg_ids: Vec<i32> = parsed_games.iter().map(|g| g.bgg_id).collect();
    let existing_games = admin_db::get_existing_games_by_bgg_ids(&db, &bgg_ids)
        .await
        .db_context("Failed to fetch existing games")?;

    // Separate into inserts and updates
    let mut games_to_insert: Vec<BggGamePreview> = Vec::new();
    let mut games_to_update: Vec<BggGameUpdatePreview> = Vec::new();

    for parsed_game in parsed_games {
        if let Some(existing) = existing_games.get(&parsed_game.bgg_id) {
            // This is an update - calculate changes
            let changes = calculate_changes(existing, &parsed_game);
            if !changes.is_empty() {
                games_to_update.push(BggGameUpdatePreview {
                    row: parsed_game.row,
                    existing_id: existing.id,
                    bgg_id: parsed_game.bgg_id,
                    name: parsed_game.name.clone(),
                    changes,
                });
            }
        } else {
            // This is a new insert
            games_to_insert.push(parsed_game.into_preview());
        }
    }

    let total_rows = (games_to_insert.len() + games_to_update.len() + errors.len()) as u32;

    success_response(BggImportPreviewResponse {
        games_to_insert,
        games_to_update,
        errors,
        total_rows,
    })
}

/// Execute BGG CSV import
#[endpoint {
    method = POST,
    path = "/api/admin/games/import"
}]
pub async fn execute_bgg_import(
    rqctx: RequestContext<AppState>,
    body: UntypedBody,
) -> Result<HttpOk<BggImportResponse>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let body_bytes = body.as_bytes();

    // Validate file size
    if body_bytes.is_empty() {
        return Err(bad_request_error("No file data provided".to_string()));
    }
    if body_bytes.len() > MAX_CSV_SIZE {
        return Err(bad_request_error(format!(
            "File too large. Maximum size is {} MB",
            MAX_CSV_SIZE / 1024 / 1024
        )));
    }

    // Parse CSV
    let (parsed_games, errors) = parse_bgg_csv(body_bytes)?;

    if parsed_games.is_empty() {
        return Err(bad_request_error("No valid games to import".to_string()));
    }

    // Get existing games by bgg_id
    let app_state = rqctx.context();
    let db = app_state.db();

    let bgg_ids: Vec<i32> = parsed_games.iter().map(|g| g.bgg_id).collect();
    let existing_games = admin_db::get_existing_games_by_bgg_ids(&db, &bgg_ids)
        .await
        .db_context("Failed to fetch existing games")?;

    // Separate into inserts and updates
    let mut games_to_insert: Vec<ParsedBggGame> = Vec::new();
    let mut games_to_update: Vec<(i64, ParsedBggGame)> = Vec::new();

    for parsed_game in parsed_games {
        if let Some(existing) = existing_games.get(&parsed_game.bgg_id) {
            games_to_update.push((existing.id, parsed_game));
        } else {
            games_to_insert.push(parsed_game);
        }
    }

    // Perform upsert
    let (inserted_count, updated_count) =
        admin_db::upsert_games_from_bgg(&db, games_to_insert, games_to_update)
            .await
            .db_context("Failed to import games")?;

    success_response(BggImportResponse {
        inserted_count,
        updated_count,
        errors,
    })
}

/// Parse BGG CSV content into games
fn parse_bgg_csv(content: &[u8]) -> Result<(Vec<ParsedBggGame>, Vec<BggParseError>), HttpError> {
    let mut reader = csv::Reader::from_reader(content);

    // Get headers to find column indices
    let headers = reader
        .headers()
        .map_err(|e| bad_request_error(format!("Invalid CSV headers: {}", e)))?
        .clone();

    let column_indices = find_column_indices(&headers)?;

    let mut parsed_games: Vec<ParsedBggGame> = Vec::new();
    let mut errors: Vec<BggParseError> = Vec::new();

    for (row_idx, result) in reader.records().enumerate() {
        let row_num = (row_idx + 2) as u32; // +2 because 1-indexed and header row

        match result {
            Ok(record) => match parse_record(&record, &column_indices, row_num) {
                Ok(game) => parsed_games.push(game),
                Err(e) => errors.push(e),
            },
            Err(e) => {
                errors.push(BggParseError {
                    row: row_num,
                    message: format!("Failed to read row: {}", e),
                });
            }
        }
    }

    Ok((parsed_games, errors))
}

/// Column indices for BGG CSV fields
struct ColumnIndices {
    objectname: usize,
    objectid: usize,
    yearpublished: Option<usize>,
    minplayers: Option<usize>,
    maxplayers: Option<usize>,
    playingtime: Option<usize>,
    avgweight: Option<usize>,
}

/// Find column indices from headers
/// Supports both BGG collection export format and BGG ranks data dump format
fn find_column_indices(headers: &csv::StringRecord) -> Result<ColumnIndices, HttpError> {
    let mut indices = HashMap::new();
    for (idx, header) in headers.iter().enumerate() {
        indices.insert(header.to_lowercase(), idx);
    }

    // Support both formats:
    // - Collection export: objectname, objectid
    // - Ranks data dump: name, id
    let objectname = indices
        .get("objectname")
        .or_else(|| indices.get("name"))
        .copied()
        .ok_or_else(|| {
            bad_request_error("Missing required column: objectname or name".to_string())
        })?;

    let objectid = indices
        .get("objectid")
        .or_else(|| indices.get("id"))
        .copied()
        .ok_or_else(|| bad_request_error("Missing required column: objectid or id".to_string()))?;

    Ok(ColumnIndices {
        objectname,
        objectid,
        yearpublished: indices.get("yearpublished").copied(),
        minplayers: indices.get("minplayers").copied(),
        maxplayers: indices.get("maxplayers").copied(),
        playingtime: indices.get("playingtime").copied(),
        avgweight: indices.get("avgweight").copied(),
    })
}

/// Parse a single CSV record into a ParsedBggGame
fn parse_record(
    record: &csv::StringRecord,
    indices: &ColumnIndices,
    row: u32,
) -> Result<ParsedBggGame, BggParseError> {
    let name = record
        .get(indices.objectname)
        .ok_or_else(|| BggParseError {
            row,
            message: "Missing objectname".to_string(),
        })?
        .trim()
        .to_string();

    if name.is_empty() {
        return Err(BggParseError {
            row,
            message: "Empty game name".to_string(),
        });
    }

    let bgg_id: i32 = record
        .get(indices.objectid)
        .ok_or_else(|| BggParseError {
            row,
            message: "Missing objectid".to_string(),
        })?
        .trim()
        .parse()
        .map_err(|_| BggParseError {
            row,
            message: "Invalid objectid (must be integer)".to_string(),
        })?;

    let year_published = indices
        .yearpublished
        .and_then(|i| record.get(i))
        .and_then(|s| s.trim().parse().ok());

    let min_players = indices
        .minplayers
        .and_then(|i| record.get(i))
        .and_then(|s| s.trim().parse().ok());

    let max_players = indices
        .maxplayers
        .and_then(|i| record.get(i))
        .and_then(|s| s.trim().parse().ok());

    let play_time_minutes = indices
        .playingtime
        .and_then(|i| record.get(i))
        .and_then(|s| s.trim().parse().ok());

    let complexity_rating = indices
        .avgweight
        .and_then(|i| record.get(i))
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|&v| (1.0..=5.0).contains(&v));

    Ok(ParsedBggGame {
        row,
        name,
        bgg_id,
        year_published,
        min_players,
        max_players,
        play_time_minutes,
        complexity_rating,
    })
}

/// Calculate field changes between existing game and new data
/// Only reports changes when the new data has a non-null value (CSV had that column)
fn calculate_changes(existing: &crate::models::Game, new_data: &ParsedBggGame) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    // Name is always required, so always check it
    if existing.name != new_data.name {
        changes.push(FieldChange {
            field: "name".to_string(),
            old_value: Some(existing.name.clone()),
            new_value: Some(new_data.name.clone()),
        });
    }

    // For optional fields, only report change if new_data has a value
    // (meaning the CSV had that column with data)
    if let Some(new_year) = new_data.year_published
        && existing.year_published != Some(new_year)
    {
        changes.push(FieldChange {
            field: "year_published".to_string(),
            old_value: existing.year_published.map(|v| v.to_string()),
            new_value: Some(new_year.to_string()),
        });
    }

    if let Some(new_min) = new_data.min_players
        && existing.min_players != Some(new_min)
    {
        changes.push(FieldChange {
            field: "min_players".to_string(),
            old_value: existing.min_players.map(|v| v.to_string()),
            new_value: Some(new_min.to_string()),
        });
    }

    if let Some(new_max) = new_data.max_players
        && existing.max_players != Some(new_max)
    {
        changes.push(FieldChange {
            field: "max_players".to_string(),
            old_value: existing.max_players.map(|v| v.to_string()),
            new_value: Some(new_max.to_string()),
        });
    }

    if let Some(new_time) = new_data.play_time_minutes
        && existing.play_time_minutes != Some(new_time)
    {
        changes.push(FieldChange {
            field: "play_time_minutes".to_string(),
            old_value: existing.play_time_minutes.map(|v| v.to_string()),
            new_value: Some(new_time.to_string()),
        });
    }

    if let Some(new_complexity) = new_data.complexity_rating {
        let complexity_changed = match existing.complexity_rating {
            Some(old) => (old - new_complexity).abs() > 0.01,
            None => true,
        };
        if complexity_changed {
            changes.push(FieldChange {
                field: "complexity_rating".to_string(),
                old_value: existing.complexity_rating.map(|v| format!("{:.2}", v)),
                new_value: Some(format!("{:.2}", new_complexity)),
            });
        }
    }

    changes
}

// ============================================================================
// BGG API Enrichment Endpoints
// ============================================================================

/// Get enrichment statistics - how many games are missing data
#[endpoint {
    method = GET,
    path = "/api/admin/bgg/stats"
}]
pub async fn get_enrichment_stats(
    rqctx: RequestContext<AppState>,
) -> Result<HttpOk<EnrichmentStats>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let app_state = rqctx.context();
    let db = app_state.db();

    let stats = admin_db::get_enrichment_stats(&db)
        .await
        .db_context("Failed to get enrichment stats")?;

    success_response(stats)
}

/// Preview BGG enrichment for a single game
#[endpoint {
    method = GET,
    path = "/api/admin/bgg/game/{id}/preview"
}]
pub async fn preview_bgg_enrich(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
) -> Result<HttpOk<BggEnrichPreviewResponse>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let game_id = path.into_inner().id;
    let app_state = rqctx.context();
    let db = app_state.db();

    // Get the game from database
    let game = admin_db::get_game_by_id(&db, game_id)
        .await
        .db_context("Failed to get game")?
        .or_not_found(format!("Game {} not found", game_id))?;

    // Check if game has BGG ID
    let bgg_id = game
        .bgg_id
        .ok_or_else(|| bad_request_error("Game does not have a BGG ID".to_string()))?;

    // Fetch data from BGG API
    let bgg_client = BggClient::new();
    let bgg_data = bgg_client
        .fetch_game(bgg_id)
        .await
        .map_err(|e| bad_request_error(format!("Failed to fetch from BGG: {}", e)))?;

    // Build current values
    let current_values = BggGameValues {
        name: game.name.clone(),
        description: game.description.clone(),
        year_published: game.year_published,
        min_players: game.min_players,
        max_players: game.max_players,
        play_time_minutes: game.play_time_minutes,
        complexity_rating: game.complexity_rating,
    };

    // Build BGG values
    let bgg_values = BggGameValues {
        name: bgg_data.name.clone(),
        description: bgg_data.description.clone(),
        year_published: bgg_data.year_published,
        min_players: bgg_data.min_players,
        max_players: bgg_data.max_players,
        play_time_minutes: bgg_data.play_time_minutes,
        complexity_rating: bgg_data.complexity_rating,
    };

    // Calculate changes
    let changes = calculate_bgg_changes(&game, &bgg_data);

    success_response(BggEnrichPreviewResponse {
        game_id,
        bgg_id,
        current_values,
        bgg_values,
        changes,
    })
}

/// Execute BGG enrichment for a single game
#[endpoint {
    method = POST,
    path = "/api/admin/bgg/game/{id}"
}]
pub async fn execute_bgg_enrich(
    rqctx: RequestContext<AppState>,
    path: Path<IdPath>,
    body: TypedBody<BggEnrichRequest>,
) -> Result<HttpOk<Game>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let game_id = path.into_inner().id;
    let request = body.into_inner();
    let app_state = rqctx.context();
    let db = app_state.db();

    // Get the game from database
    let game = admin_db::get_game_by_id(&db, game_id)
        .await
        .db_context("Failed to get game")?
        .or_not_found(format!("Game {} not found", game_id))?;

    // Check if game has BGG ID
    let bgg_id = game
        .bgg_id
        .ok_or_else(|| bad_request_error("Game does not have a BGG ID".to_string()))?;

    if request.fields_to_update.is_empty() {
        return Err(bad_request_error(
            "No fields selected for update".to_string(),
        ));
    }

    // Fetch data from BGG API
    let bgg_client = BggClient::new();
    let bgg_data = bgg_client
        .fetch_game(bgg_id)
        .await
        .map_err(|e| bad_request_error(format!("Failed to fetch from BGG: {}", e)))?;

    // Update the game
    let updated_game =
        admin_db::update_game_from_bgg(&db, game_id, &bgg_data, &request.fields_to_update)
            .await
            .db_context("Failed to update game")?;

    success_response(updated_game)
}

/// Preview bulk BGG enrichment
#[endpoint {
    method = POST,
    path = "/api/admin/bgg/bulk/preview"
}]
pub async fn preview_bulk_enrich(
    rqctx: RequestContext<AppState>,
    body: TypedBody<BulkEnrichRequest>,
) -> Result<HttpOk<BulkEnrichPreviewResponse>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let request = body.into_inner();
    let limit = request.limit.unwrap_or(50).min(200); // Cap at 200 for preview
    let app_state = rqctx.context();
    let db = app_state.db();

    // Get games needing enrichment
    let games = admin_db::get_games_needing_enrichment(&db, limit)
        .await
        .db_context("Failed to get games")?;

    if games.is_empty() {
        return success_response(BulkEnrichPreviewResponse {
            games_to_update: Vec::new(),
            errors: Vec::new(),
            total_fetched: 0,
        });
    }

    // Collect BGG IDs
    let bgg_ids: Vec<i32> = games.iter().filter_map(|g| g.bgg_id).collect();

    // Fetch from BGG API (batched with rate limiting)
    let bgg_client = BggClient::new();
    let bgg_results = bgg_client
        .fetch_games(&bgg_ids)
        .await
        .map_err(|e| bad_request_error(format!("Failed to fetch from BGG: {}", e)))?;

    // Build lookup map by BGG ID
    let bgg_map: HashMap<i32, _> = bgg_results.into_iter().map(|g| (g.bgg_id, g)).collect();

    // Build preview
    let mut games_to_update = Vec::new();
    let mut errors = Vec::new();

    for game in &games {
        let bgg_id = match game.bgg_id {
            Some(id) => id,
            None => continue,
        };

        match bgg_map.get(&bgg_id) {
            Some(bgg_data) => {
                let changes = calculate_bgg_changes(game, bgg_data);
                // Only include if there are actual changes for requested fields
                let filtered_changes: Vec<_> = changes
                    .into_iter()
                    .filter(|c| request.fields_to_enrich.contains(&c.field))
                    .collect();

                if !filtered_changes.is_empty() {
                    games_to_update.push(BggGameEnrichPreview {
                        game_id: game.id,
                        bgg_id,
                        name: game.name.clone(),
                        changes: filtered_changes,
                    });
                }
            }
            None => {
                errors.push(BggEnrichError {
                    game_id: game.id,
                    bgg_id,
                    message: "Game not found on BGG".to_string(),
                });
            }
        }
    }

    success_response(BulkEnrichPreviewResponse {
        games_to_update,
        errors,
        total_fetched: bgg_map.len() as u32,
    })
}

/// Execute bulk BGG enrichment
#[endpoint {
    method = POST,
    path = "/api/admin/bgg/bulk"
}]
pub async fn execute_bulk_enrich(
    rqctx: RequestContext<AppState>,
    body: TypedBody<BulkEnrichRequest>,
) -> Result<HttpOk<BulkEnrichResponse>, HttpError> {
    // Verify admin access
    require_admin(&rqctx)?;

    let request = body.into_inner();
    let limit = request.limit.unwrap_or(50).min(200);
    let app_state = rqctx.context();
    let db = app_state.db();

    if request.fields_to_enrich.is_empty() {
        return Err(bad_request_error(
            "No fields selected for enrichment".to_string(),
        ));
    }

    // Get games needing enrichment
    let games = admin_db::get_games_needing_enrichment(&db, limit)
        .await
        .db_context("Failed to get games")?;

    if games.is_empty() {
        return success_response(BulkEnrichResponse {
            updated_count: 0,
            errors: Vec::new(),
        });
    }

    // Collect BGG IDs
    let bgg_ids: Vec<i32> = games.iter().filter_map(|g| g.bgg_id).collect();

    // Fetch from BGG API
    let bgg_client = BggClient::new();
    let bgg_results = bgg_client
        .fetch_games(&bgg_ids)
        .await
        .map_err(|e| bad_request_error(format!("Failed to fetch from BGG: {}", e)))?;

    // Build lookup map
    let bgg_map: HashMap<i32, _> = bgg_results.into_iter().map(|g| (g.bgg_id, g)).collect();

    // Build updates list
    let mut updates = Vec::new();
    let mut errors = Vec::new();

    for game in &games {
        let bgg_id = match game.bgg_id {
            Some(id) => id,
            None => continue,
        };

        match bgg_map.get(&bgg_id) {
            Some(bgg_data) => {
                updates.push((game.id, bgg_data.clone(), request.fields_to_enrich.clone()));
            }
            None => {
                errors.push(BggEnrichError {
                    game_id: game.id,
                    bgg_id,
                    message: "Game not found on BGG".to_string(),
                });
            }
        }
    }

    // Execute batch update
    let updated_count = admin_db::batch_update_games_from_bgg(&db, updates)
        .await
        .db_context("Failed to update games")?;

    success_response(BulkEnrichResponse {
        updated_count,
        errors,
    })
}

/// Calculate changes between existing game and BGG data
fn calculate_bgg_changes(existing: &Game, bgg_data: &crate::bgg::BggGameData) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    // Description
    if let Some(ref new_desc) = bgg_data.description {
        let desc_changed = match &existing.description {
            Some(old) => old != new_desc,
            None => true,
        };
        if desc_changed {
            changes.push(FieldChange {
                field: "description".to_string(),
                old_value: existing
                    .description
                    .as_ref()
                    .map(|s| truncate_for_display(s, 100)),
                new_value: Some(truncate_for_display(new_desc, 100)),
            });
        }
    }

    // Year published
    if let Some(new_year) = bgg_data.year_published
        && existing.year_published != Some(new_year)
    {
        changes.push(FieldChange {
            field: "year_published".to_string(),
            old_value: existing.year_published.map(|v| v.to_string()),
            new_value: Some(new_year.to_string()),
        });
    }

    // Min players
    if let Some(new_min) = bgg_data.min_players
        && existing.min_players != Some(new_min)
    {
        changes.push(FieldChange {
            field: "min_players".to_string(),
            old_value: existing.min_players.map(|v| v.to_string()),
            new_value: Some(new_min.to_string()),
        });
    }

    // Max players
    if let Some(new_max) = bgg_data.max_players
        && existing.max_players != Some(new_max)
    {
        changes.push(FieldChange {
            field: "max_players".to_string(),
            old_value: existing.max_players.map(|v| v.to_string()),
            new_value: Some(new_max.to_string()),
        });
    }

    // Play time
    if let Some(new_time) = bgg_data.play_time_minutes
        && existing.play_time_minutes != Some(new_time)
    {
        changes.push(FieldChange {
            field: "play_time_minutes".to_string(),
            old_value: existing.play_time_minutes.map(|v| v.to_string()),
            new_value: Some(new_time.to_string()),
        });
    }

    // Complexity rating
    if let Some(new_complexity) = bgg_data.complexity_rating {
        let complexity_changed = match existing.complexity_rating {
            Some(old) => (old - new_complexity).abs() > 0.01,
            None => true,
        };
        if complexity_changed {
            changes.push(FieldChange {
                field: "complexity_rating".to_string(),
                old_value: existing.complexity_rating.map(|v| format!("{:.2}", v)),
                new_value: Some(format!("{:.2}", new_complexity)),
            });
        }
    }

    changes
}

/// Truncate a string for display purposes
fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
