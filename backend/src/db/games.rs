use super::{
    Database, PaginationInfo, format_now_for_db, like_pattern, parse_datetime, query_row_optional,
};
use crate::models::{
    CreateGameRequest, Game, GameSummary, PaginatedResponse, RulesInfoResponse, UpdateGameRequest,
};
use rusqlite::{Result as SqliteResult, Row, params};

/// Map a database row to a Game struct
fn row_to_game(row: &Row) -> SqliteResult<Game> {
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
        created_at: parse_datetime(row, "created_at")?,
        updated_at: parse_datetime(row, "updated_at")?,
    })
}

fn row_to_game_summary(row: &Row) -> SqliteResult<GameSummary> {
    Ok(GameSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        publisher: row.get(2)?,
        year_published: row.get(3)?,
        min_players: row.get(4)?,
        max_players: row.get(5)?,
        complexity_rating: row.get(6)?,
        has_rules_pdf: row.get::<_, Option<String>>(7)?.is_some(),
        house_rules_count: row.get(8)?,
    })
}

pub async fn list_games(
    db: &Database,
    page: u32,
    limit: u32,
    search: Option<&str>,
    has_rules_pdf: Option<bool>,
) -> SqliteResult<PaginatedResponse<GameSummary>> {
    let pagination = PaginationInfo::new(page, limit);

    db.with_connection(|conn| {
        // Build WHERE clauses
        let search_pattern = search.map(like_pattern);
        let mut where_conditions: Vec<String> = Vec::new();

        if search_pattern.is_some() {
            where_conditions.push("LOWER(g.name) LIKE ? ESCAPE '\\'".to_string());
        }

        if let Some(has_pdf) = has_rules_pdf {
            if has_pdf {
                where_conditions.push("g.rules_pdf_path IS NOT NULL".to_string());
            } else {
                where_conditions.push("g.rules_pdf_path IS NULL".to_string());
            }
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Get total count (with filters)
        let count_query = format!("SELECT COUNT(*) FROM master_games g {}", where_clause);
        let total: u32 = if let Some(ref pattern) = search_pattern {
            conn.query_row(&count_query, params![pattern], |row| row.get(0))?
        } else {
            conn.query_row(&count_query, [], |row| row.get(0))?
        };

        // Get games with house rules count
        let query = format!(
            r#"
            SELECT
                g.id, g.name, g.publisher, g.year_published,
                g.min_players, g.max_players, g.complexity_rating,
                g.rules_pdf_path,
                COUNT(hr.id) as house_rules_count
            FROM master_games g
            LEFT JOIN house_rules hr ON g.id = hr.game_id AND hr.is_active = TRUE
            {}
            GROUP BY g.id, g.name, g.publisher, g.year_published,
                     g.min_players, g.max_players, g.complexity_rating, g.rules_pdf_path
            ORDER BY g.name ASC
            LIMIT ? OFFSET ?
            "#,
            where_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let games: Vec<GameSummary> = if let Some(ref pattern) = search_pattern {
            stmt.query_map(
                params![pattern, pagination.limit, pagination.offset],
                row_to_game_summary,
            )?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(
                params![pagination.limit, pagination.offset],
                row_to_game_summary,
            )?
            .collect::<Result<Vec<_>, _>>()?
        };

        Ok(PaginatedResponse::new(games, total, page, limit))
    })
}

pub async fn get_game(db: &Database, game_id: i64) -> SqliteResult<Option<Game>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, publisher, year_published,
                   min_players, max_players, play_time_minutes, complexity_rating,
                   bgg_id, rules_pdf_path, rules_text, created_at, updated_at
            FROM master_games WHERE id = ?
            "#,
        )?;

        query_row_optional(stmt.query_row(params![game_id], row_to_game))
    })
}

pub async fn create_game(db: &Database, request: CreateGameRequest) -> SqliteResult<Game> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        conn.execute(
            r#"
            INSERT INTO master_games (
                name, description, publisher, year_published,
                min_players, max_players, play_time_minutes, complexity_rating,
                bgg_id, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                request.name,
                request.description,
                request.publisher,
                request.year_published,
                request.min_players,
                request.max_players,
                request.play_time_minutes,
                request.complexity_rating,
                request.bgg_id,
                now_str,
                now_str
            ],
        )?;

        let game_id = conn.last_insert_rowid();

        // Fetch the created game
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, publisher, year_published,
                   min_players, max_players, play_time_minutes, complexity_rating,
                   bgg_id, rules_pdf_path, rules_text, created_at, updated_at
            FROM master_games WHERE id = ?
            "#,
        )?;

        stmt.query_row(params![game_id], row_to_game)
    })
}

pub async fn update_game(
    db: &Database,
    game_id: i64,
    request: UpdateGameRequest,
) -> SqliteResult<Option<Game>> {
    db.with_transaction(|conn| {
        // Check if game exists
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM master_games WHERE id = ?)",
            params![game_id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(None);
        }

        let now_str = format_now_for_db();

        // Build dynamic update query
        let mut update_parts = Vec::new();
        let mut params_vec = Vec::new();

        if let Some(name) = &request.name {
            update_parts.push("name = ?");
            params_vec.push(name as &dyn rusqlite::ToSql);
        }
        if let Some(description) = &request.description {
            update_parts.push("description = ?");
            params_vec.push(description as &dyn rusqlite::ToSql);
        }
        if let Some(publisher) = &request.publisher {
            update_parts.push("publisher = ?");
            params_vec.push(publisher as &dyn rusqlite::ToSql);
        }
        if let Some(year_published) = &request.year_published {
            update_parts.push("year_published = ?");
            params_vec.push(year_published as &dyn rusqlite::ToSql);
        }
        if let Some(min_players) = &request.min_players {
            update_parts.push("min_players = ?");
            params_vec.push(min_players as &dyn rusqlite::ToSql);
        }
        if let Some(max_players) = &request.max_players {
            update_parts.push("max_players = ?");
            params_vec.push(max_players as &dyn rusqlite::ToSql);
        }
        if let Some(play_time_minutes) = &request.play_time_minutes {
            update_parts.push("play_time_minutes = ?");
            params_vec.push(play_time_minutes as &dyn rusqlite::ToSql);
        }
        if let Some(complexity_rating) = &request.complexity_rating {
            update_parts.push("complexity_rating = ?");
            params_vec.push(complexity_rating as &dyn rusqlite::ToSql);
        }
        if let Some(bgg_id) = &request.bgg_id {
            update_parts.push("bgg_id = ?");
            params_vec.push(bgg_id as &dyn rusqlite::ToSql);
        }

        if update_parts.is_empty() {
            // No updates requested, just return the current game
            return get_game_by_id_sync(conn, game_id).map(Some);
        }

        update_parts.push("updated_at = ?");
        params_vec.push(&now_str as &dyn rusqlite::ToSql);
        params_vec.push(&game_id as &dyn rusqlite::ToSql);

        let query = format!(
            "UPDATE master_games SET {} WHERE id = ?",
            update_parts.join(", ")
        );

        conn.execute(&query, params_vec.as_slice())?;

        get_game_by_id_sync(conn, game_id).map(Some)
    })
}

pub async fn delete_game(db: &Database, game_id: i64) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows_affected =
            conn.execute("DELETE FROM master_games WHERE id = ?", params![game_id])?;
        Ok(rows_affected > 0)
    })
}

pub async fn update_game_rules_text(
    db: &Database,
    game_id: i64,
    rules_text: String,
    pdf_path: Option<String>,
) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let now_str = format_now_for_db();
        let rows_affected = conn.execute(
            "UPDATE master_games SET rules_text = ?, rules_pdf_path = ?, updated_at = ? WHERE id = ?",
            params![rules_text, pdf_path, now_str, game_id],
        )?;
        Ok(rows_affected > 0)
    })
}

pub async fn get_game_rules_info(
    db: &Database,
    game_id: i64,
) -> SqliteResult<Option<RulesInfoResponse>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                g.name,
                g.rules_pdf_path,
                g.rules_text,
                COUNT(e.id) as chunk_count,
                MAX(e.created_at) as last_processed
            FROM master_games g
            LEFT JOIN embeddings e ON g.id = e.game_id AND e.source_type = 'rules_pdf'
            WHERE g.id = ?
            GROUP BY g.id
            "#,
        )?;

        query_row_optional(stmt.query_row(params![game_id], |row| {
            Ok(RulesInfoResponse {
                game_id,
                game_name: row.get(0)?,
                has_rules_pdf: row.get::<_, Option<String>>(1)?.is_some(),
                rules_pdf_path: row.get(1)?,
                text_length: row.get::<_, Option<String>>(2)?.map(|s| s.len()),
                chunk_count: row.get(3)?,
                last_processed: row.get(4)?,
            })
        }))
    })
}

// Helper function for synchronous game retrieval within transactions
fn get_game_by_id_sync(conn: &rusqlite::Connection, game_id: i64) -> SqliteResult<Game> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, description, publisher, year_published,
               min_players, max_players, play_time_minutes, complexity_rating,
               bgg_id, rules_pdf_path, rules_text, created_at, updated_at
        FROM master_games WHERE id = ?
        "#,
    )?;

    stmt.query_row(params![game_id], row_to_game)
}
