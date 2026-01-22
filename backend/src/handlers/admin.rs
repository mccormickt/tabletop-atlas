use crate::{
    AppState,
    auth::middleware::require_admin,
    db::admin as admin_db,
    handlers::{bad_request_error, forbidden_error, internal_error, success_response},
    models::admin::{
        BggGamePreview, BggGameUpdatePreview, BggImportPreviewResponse, BggImportResponse,
        BggParseError, FieldChange, ParsedBggGame,
    },
};
use dropshot::{RequestContext, UntypedBody, endpoint};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;

use super::{HttpError, HttpOk};

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
    require_admin(&rqctx).map_err(|e| forbidden_error(e.external_message.clone()))?;

    let app_state = rqctx.context();
    let db = app_state.db();

    let master_games_count = admin_db::get_master_games_count(&db)
        .await
        .map_err(|e| internal_error(format!("Failed to get stats: {}", e)))?;

    success_response(AdminDashboardStats { master_games_count })
}

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
    require_admin(&rqctx).map_err(|e| forbidden_error(e.external_message.clone()))?;

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
        .map_err(|e| internal_error(format!("Failed to fetch existing games: {}", e)))?;

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
    require_admin(&rqctx).map_err(|e| forbidden_error(e.external_message.clone()))?;

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
        .map_err(|e| internal_error(format!("Failed to fetch existing games: {}", e)))?;

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
            .map_err(|e| internal_error(format!("Failed to import games: {}", e)))?;

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
    if let Some(new_year) = new_data.year_published {
        if existing.year_published != Some(new_year) {
            changes.push(FieldChange {
                field: "year_published".to_string(),
                old_value: existing.year_published.map(|v| v.to_string()),
                new_value: Some(new_year.to_string()),
            });
        }
    }

    if let Some(new_min) = new_data.min_players {
        if existing.min_players != Some(new_min) {
            changes.push(FieldChange {
                field: "min_players".to_string(),
                old_value: existing.min_players.map(|v| v.to_string()),
                new_value: Some(new_min.to_string()),
            });
        }
    }

    if let Some(new_max) = new_data.max_players {
        if existing.max_players != Some(new_max) {
            changes.push(FieldChange {
                field: "max_players".to_string(),
                old_value: existing.max_players.map(|v| v.to_string()),
                new_value: Some(new_max.to_string()),
            });
        }
    }

    if let Some(new_time) = new_data.play_time_minutes {
        if existing.play_time_minutes != Some(new_time) {
            changes.push(FieldChange {
                field: "play_time_minutes".to_string(),
                old_value: existing.play_time_minutes.map(|v| v.to_string()),
                new_value: Some(new_time.to_string()),
            });
        }
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
