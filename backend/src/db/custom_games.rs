use super::{Database, PaginatedQuery, format_now_for_db, parse_datetime, query_row_optional};
use crate::models::{
    CreateCustomGameRequest, CustomGame, CustomGameSummary, PaginatedResponse,
    UpdateCustomGameRequest,
};
use rusqlite::{Result as SqliteResult, Row, params};

/// Map a database row to a CustomGame struct
fn row_to_custom_game(row: &Row) -> SqliteResult<CustomGame> {
    Ok(CustomGame {
        id: row.get(0)?,
        user_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        publisher: row.get(4)?,
        year_published: row.get(5)?,
        min_players: row.get(6)?,
        max_players: row.get(7)?,
        play_time_minutes: row.get(8)?,
        complexity_rating: row.get(9)?,
        rules_pdf_path: row.get(10)?,
        rules_text: row.get(11)?,
        is_public: row.get::<_, i32>(12)? != 0,
        created_at: parse_datetime(row, "created_at")?,
        updated_at: parse_datetime(row, "updated_at")?,
    })
}

fn row_to_custom_game_summary(row: &Row) -> SqliteResult<CustomGameSummary> {
    Ok(CustomGameSummary {
        id: row.get(0)?,
        user_id: row.get(1)?,
        name: row.get(2)?,
        publisher: row.get(3)?,
        year_published: row.get(4)?,
        min_players: row.get(5)?,
        max_players: row.get(6)?,
        complexity_rating: row.get(7)?,
        is_public: row.get::<_, i32>(8)? != 0,
        has_rules_pdf: row.get::<_, Option<String>>(9)?.is_some(),
    })
}

pub async fn list_user_custom_games(
    db: &Database,
    user_id: i64,
    page: u32,
    limit: u32,
) -> SqliteResult<PaginatedResponse<CustomGameSummary>> {
    db.with_connection(|conn| {
        let mut q = PaginatedQuery::new();
        q.filter("user_id = ?", user_id);

        q.execute(
            conn,
            "custom_games",
            "id, user_id, name, publisher, year_published, min_players, max_players, complexity_rating, is_public, rules_pdf_path",
            "custom_games",
            "name ASC",
            None,
            page,
            limit,
            row_to_custom_game_summary,
        )
    })
}

pub async fn list_public_custom_games(
    db: &Database,
    page: u32,
    limit: u32,
) -> SqliteResult<PaginatedResponse<CustomGameSummary>> {
    db.with_connection(|conn| {
        let mut q = PaginatedQuery::new();
        q.filter_raw("is_public = 1");

        q.execute(
            conn,
            "custom_games",
            "id, user_id, name, publisher, year_published, min_players, max_players, complexity_rating, is_public, rules_pdf_path",
            "custom_games",
            "name ASC",
            None,
            page,
            limit,
            row_to_custom_game_summary,
        )
    })
}

pub async fn create_custom_game(
    db: &Database,
    user_id: i64,
    request: CreateCustomGameRequest,
) -> SqliteResult<CustomGame> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();
        let is_public = if request.is_public.unwrap_or(false) {
            1
        } else {
            0
        };

        conn.execute(
            r#"
            INSERT INTO custom_games (
                user_id, name, description, publisher, year_published,
                min_players, max_players, play_time_minutes, complexity_rating,
                is_public, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                user_id,
                request.name,
                request.description,
                request.publisher,
                request.year_published,
                request.min_players,
                request.max_players,
                request.play_time_minutes,
                request.complexity_rating,
                is_public,
                now_str,
                now_str
            ],
        )?;

        let game_id = conn.last_insert_rowid();
        get_custom_game_by_id_sync(conn, game_id)
    })
}

pub async fn get_custom_game(db: &Database, game_id: i64) -> SqliteResult<Option<CustomGame>> {
    db.with_connection(|conn| query_row_optional(get_custom_game_by_id_sync(conn, game_id)))
}

pub async fn update_custom_game(
    db: &Database,
    user_id: i64,
    game_id: i64,
    request: UpdateCustomGameRequest,
) -> SqliteResult<Option<CustomGame>> {
    db.with_transaction(|conn| {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM custom_games WHERE id = ? AND user_id = ?)",
            params![game_id, user_id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(None);
        }

        let now_str = format_now_for_db();

        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(name) = request.name {
            update_parts.push("name = ?");
            params_vec.push(Box::new(name));
        }
        if let Some(description) = request.description {
            update_parts.push("description = ?");
            params_vec.push(Box::new(description));
        }
        if let Some(publisher) = request.publisher {
            update_parts.push("publisher = ?");
            params_vec.push(Box::new(publisher));
        }
        if let Some(year_published) = request.year_published {
            update_parts.push("year_published = ?");
            params_vec.push(Box::new(year_published));
        }
        if let Some(min_players) = request.min_players {
            update_parts.push("min_players = ?");
            params_vec.push(Box::new(min_players));
        }
        if let Some(max_players) = request.max_players {
            update_parts.push("max_players = ?");
            params_vec.push(Box::new(max_players));
        }
        if let Some(play_time_minutes) = request.play_time_minutes {
            update_parts.push("play_time_minutes = ?");
            params_vec.push(Box::new(play_time_minutes));
        }
        if let Some(complexity_rating) = request.complexity_rating {
            update_parts.push("complexity_rating = ?");
            params_vec.push(Box::new(complexity_rating));
        }
        if let Some(is_public) = request.is_public {
            update_parts.push("is_public = ?");
            params_vec.push(Box::new(if is_public { 1 } else { 0 }));
        }

        if !update_parts.is_empty() {
            update_parts.push("updated_at = ?");
            params_vec.push(Box::new(now_str));
            params_vec.push(Box::new(game_id));

            let query = format!(
                "UPDATE custom_games SET {} WHERE id = ?",
                update_parts.join(", ")
            );
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&query, params_refs.as_slice())?;
        }

        Ok(Some(get_custom_game_by_id_sync(conn, game_id)?))
    })
}

pub async fn delete_custom_game(db: &Database, user_id: i64, game_id: i64) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute(
            "DELETE FROM custom_games WHERE id = ? AND user_id = ?",
            params![game_id, user_id],
        )?;
        Ok(rows_affected > 0)
    })
}

fn get_custom_game_by_id_sync(
    conn: &rusqlite::Connection,
    game_id: i64,
) -> SqliteResult<CustomGame> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, user_id, name, description, publisher, year_published,
               min_players, max_players, play_time_minutes, complexity_rating,
               rules_pdf_path, rules_text, is_public, created_at, updated_at
        FROM custom_games WHERE id = ?
        "#,
    )?;

    stmt.query_row(params![game_id], row_to_custom_game)
}
