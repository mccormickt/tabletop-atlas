use super::{Database, PaginationInfo, format_now_for_db, parse_datetime};
use crate::models::{
    AddToCollectionRequest, CollectionEntry, CollectionEntryId, CollectionEntryWithGame,
    PaginatedResponse, UpdateCollectionRequest,
};
use rusqlite::{Result as SqliteResult, Row, params};

/// Map a database row to a CollectionEntry struct
fn row_to_collection_entry(row: &Row) -> SqliteResult<CollectionEntry> {
    Ok(CollectionEntry {
        id: row.get(0)?,
        user_id: row.get(1)?,
        master_game_id: row.get(2)?,
        notes: row.get(3)?,
        rating: row.get(4)?,
        play_count: row.get(5)?,
        added_at: parse_datetime(row, "added_at")?,
    })
}

pub async fn list_user_collection(
    db: &Database,
    user_id: i64,
    page: u32,
    limit: u32,
) -> SqliteResult<PaginatedResponse<CollectionEntryWithGame>> {
    let pagination = PaginationInfo::new(page, limit);

    db.with_connection(|conn| {
        let total: u32 = conn.query_row(
            "SELECT COUNT(*) FROM user_collections WHERE user_id = ?",
            params![user_id],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            r#"
            SELECT uc.id, uc.master_game_id, mg.name, uc.notes, uc.rating, uc.play_count, uc.added_at
            FROM user_collections uc
            JOIN master_games mg ON uc.master_game_id = mg.id
            WHERE uc.user_id = ?
            ORDER BY uc.added_at DESC
            LIMIT ? OFFSET ?
            "#,
        )?;

        let entries = stmt
            .query_map(params![user_id, pagination.limit, pagination.offset], |row| {
                Ok(CollectionEntryWithGame {
                    id: row.get(0)?,
                    master_game_id: row.get(1)?,
                    game_name: row.get(2)?,
                    notes: row.get(3)?,
                    rating: row.get(4)?,
                    play_count: row.get(5)?,
                    added_at: parse_datetime(row, "added_at")?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResponse::new(entries, total, page, limit))
    })
}

pub async fn add_to_collection(
    db: &Database,
    user_id: i64,
    request: AddToCollectionRequest,
) -> SqliteResult<CollectionEntry> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        conn.execute(
            r#"
            INSERT INTO user_collections (user_id, master_game_id, notes, rating, added_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            params![
                user_id,
                request.master_game_id,
                request.notes,
                request.rating,
                now_str
            ],
        )?;

        let entry_id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, user_id, master_game_id, notes, rating, play_count, added_at
            FROM user_collections WHERE id = ?
            "#,
        )?;

        stmt.query_row(params![entry_id], row_to_collection_entry)
    })
}

pub async fn update_collection_entry(
    db: &Database,
    user_id: i64,
    entry_id: CollectionEntryId,
    request: UpdateCollectionRequest,
) -> SqliteResult<Option<CollectionEntry>> {
    db.with_transaction(|conn| {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM user_collections WHERE id = ? AND user_id = ?)",
            params![entry_id, user_id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(None);
        }

        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(notes) = request.notes {
            update_parts.push("notes = ?");
            params_vec.push(Box::new(notes));
        }
        if let Some(rating) = request.rating {
            update_parts.push("rating = ?");
            params_vec.push(Box::new(rating));
        }
        if let Some(play_count) = request.play_count {
            update_parts.push("play_count = ?");
            params_vec.push(Box::new(play_count));
        }

        if !update_parts.is_empty() {
            params_vec.push(Box::new(entry_id));
            let query = format!(
                "UPDATE user_collections SET {} WHERE id = ?",
                update_parts.join(", ")
            );
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&query, params_refs.as_slice())?;
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT id, user_id, master_game_id, notes, rating, play_count, added_at
            FROM user_collections WHERE id = ?
            "#,
        )?;

        Ok(Some(
            stmt.query_row(params![entry_id], row_to_collection_entry)?,
        ))
    })
}

pub async fn remove_from_collection(
    db: &Database,
    user_id: i64,
    entry_id: CollectionEntryId,
) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute(
            "DELETE FROM user_collections WHERE id = ? AND user_id = ?",
            params![entry_id, user_id],
        )?;
        Ok(rows_affected > 0)
    })
}
