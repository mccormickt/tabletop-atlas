use super::{Database, format_now_for_db};
use crate::models::Game;
use crate::models::admin::ParsedBggGame;
use rusqlite::{Result as SqliteResult, params};
use std::collections::HashMap;

// Batch size for queries to stay under SQLite's parameter limit (~999)
const QUERY_BATCH_SIZE: usize = 500;

/// Get existing games by their BGG IDs
/// Batches queries to avoid SQLite parameter limits
pub async fn get_existing_games_by_bgg_ids(
    db: &Database,
    bgg_ids: &[i32],
) -> SqliteResult<HashMap<i32, Game>> {
    if bgg_ids.is_empty() {
        return Ok(HashMap::new());
    }

    db.with_connection(|conn| {
        let mut result = HashMap::new();

        // Process in batches to stay under SQLite's parameter limit
        for batch in bgg_ids.chunks(QUERY_BATCH_SIZE) {
            let placeholders: String = batch.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                r#"
                SELECT id, name, description, publisher, year_published,
                       min_players, max_players, play_time_minutes, complexity_rating,
                       bgg_id, rules_pdf_path, rules_text, created_at, updated_at
                FROM master_games
                WHERE bgg_id IN ({})
                "#,
                placeholders
            );

            let mut stmt = conn.prepare(&query)?;
            let params: Vec<&dyn rusqlite::ToSql> = batch
                .iter()
                .map(|id| id as &dyn rusqlite::ToSql)
                .collect();

            let game_iter = stmt.query_map(params.as_slice(), |row| {
                Ok(Game {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    publisher: row.get(3)?,
                    year_published: row.get(4)?,
                    min_players: row.get(5)?,
                    max_players: row.get(6)?,
                    play_time_minutes: row.get(7)?,
                    complexity_rating: row.get(8)?,
                    bgg_id: row.get(9)?,
                    rules_pdf_path: row.get(10)?,
                    rules_text: row.get(11)?,
                    created_at: super::parse_datetime(row, "created_at")?,
                    updated_at: super::parse_datetime(row, "updated_at")?,
                })
            })?;

            for game in game_iter {
                let game = game?;
                if let Some(bgg_id) = game.bgg_id {
                    result.insert(bgg_id, game);
                }
            }
        }

        Ok(result)
    })
}

// Batch size for inserts/updates to stay under SQLite's parameter limit
// SQLite has a limit of ~999 parameters per query
// With 9 parameters per insert, 100 records per batch is safe
const BATCH_SIZE: usize = 100;

/// Upsert games from BGG import (insert new, update existing by bgg_id)
/// Preserves rules_pdf_path and rules_text on update
/// Batches operations to avoid SQLite parameter limits
pub async fn upsert_games_from_bgg(
    db: &Database,
    games_to_insert: Vec<ParsedBggGame>,
    games_to_update: Vec<(i64, ParsedBggGame)>, // (existing_id, new_data)
) -> SqliteResult<(u32, u32)> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();
        let mut inserted = 0u32;
        let mut updated = 0u32;

        // Prepare statements once, reuse for each record
        let mut insert_stmt = conn.prepare(
            r#"
            INSERT INTO master_games (
                name, year_published, min_players, max_players,
                play_time_minutes, complexity_rating, bgg_id,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;

        // Use COALESCE to preserve existing values when import has null
        let mut update_stmt = conn.prepare(
            r#"
            UPDATE master_games SET
                name = ?,
                year_published = COALESCE(?, year_published),
                min_players = COALESCE(?, min_players),
                max_players = COALESCE(?, max_players),
                play_time_minutes = COALESCE(?, play_time_minutes),
                complexity_rating = COALESCE(?, complexity_rating),
                updated_at = ?
            WHERE id = ?
            "#,
        )?;

        // Insert new games in batches
        for batch in games_to_insert.chunks(BATCH_SIZE) {
            for game in batch {
                insert_stmt.execute(params![
                    game.name,
                    game.year_published,
                    game.min_players,
                    game.max_players,
                    game.play_time_minutes,
                    game.complexity_rating,
                    game.bgg_id,
                    now_str,
                    now_str
                ])?;
                inserted += 1;
            }
        }

        // Update existing games in batches (preserve rules_pdf_path and rules_text)
        for batch in games_to_update.chunks(BATCH_SIZE) {
            for (existing_id, game) in batch {
                update_stmt.execute(params![
                    game.name,
                    game.year_published,
                    game.min_players,
                    game.max_players,
                    game.play_time_minutes,
                    game.complexity_rating,
                    now_str,
                    existing_id
                ])?;
                updated += 1;
            }
        }

        Ok((inserted, updated))
    })
}

/// Get count of master games
pub async fn get_master_games_count(db: &Database) -> SqliteResult<u32> {
    db.with_connection(|conn| {
        conn.query_row("SELECT COUNT(*) FROM master_games", [], |row| row.get(0))
    })
}
