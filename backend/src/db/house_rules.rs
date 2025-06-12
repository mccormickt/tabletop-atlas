use rusqlite::{params, Result as SqliteResult};
use chrono::Utc;
use crate::models::{HouseRule, HouseRuleId, GameId, CreateHouseRuleRequest, UpdateHouseRuleRequest, PaginatedResponse};
use super::{Database, parse_datetime, PaginationInfo};

pub async fn list_house_rules(db: &Database, game_id: GameId, page: u32, limit: u32) -> SqliteResult<PaginatedResponse<HouseRule>> {
    let pagination = PaginationInfo::new(page, limit);
    
    db.with_connection(|conn| {
        // Get total count for the specific game
        let total: u32 = conn.query_row(
            "SELECT COUNT(*) FROM house_rules WHERE game_id = ?",
            params![game_id],
            |row| row.get(0)
        )?;

        // Get house rules for the game
        let mut stmt = conn.prepare(
            r#"
            SELECT id, game_id, title, description, category, is_active, created_at, updated_at
            FROM house_rules 
            WHERE game_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#
        )?;

        let house_rule_iter = stmt.query_map(params![game_id, pagination.limit, pagination.offset], |row| {
            Ok(HouseRule {
                id: row.get(0)?,
                game_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                is_active: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        })?;

        let house_rules: Result<Vec<HouseRule>, _> = house_rule_iter.collect();
        let house_rules = house_rules?;

        Ok(PaginatedResponse::new(house_rules, total, page, limit))
    })
}

pub async fn get_house_rule(db: &Database, house_rule_id: HouseRuleId) -> SqliteResult<Option<HouseRule>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, game_id, title, description, category, is_active, created_at, updated_at
            FROM house_rules WHERE id = ?
            "#
        )?;

        let result = stmt.query_row(params![house_rule_id], |row| {
            Ok(HouseRule {
                id: row.get(0)?,
                game_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                is_active: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        });

        match result {
            Ok(house_rule) => Ok(Some(house_rule)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    })
}

pub async fn create_house_rule(db: &Database, request: CreateHouseRuleRequest) -> SqliteResult<HouseRule> {
    db.with_transaction(|conn| {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        // First verify the game exists
        let game_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?)",
            params![request.game_id],
            |row| row.get(0)
        )?;

        if !game_exists {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some("Game does not exist".to_string())
            ));
        }

        conn.execute(
            r#"
            INSERT INTO house_rules (
                game_id, title, description, category, is_active, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                request.game_id,
                request.title,
                request.description,
                request.category,
                request.is_active,
                now_str,
                now_str
            ]
        )?;

        let house_rule_id = conn.last_insert_rowid();

        // Fetch the created house rule
        let mut stmt = conn.prepare(
            r#"
            SELECT id, game_id, title, description, category, is_active, created_at, updated_at
            FROM house_rules WHERE id = ?
            "#
        )?;

        stmt.query_row(params![house_rule_id], |row| {
            Ok(HouseRule {
                id: row.get(0)?,
                game_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                is_active: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        })
    })
}

pub async fn update_house_rule(db: &Database, house_rule_id: HouseRuleId, request: UpdateHouseRuleRequest) -> SqliteResult<Option<HouseRule>> {
    db.with_transaction(|conn| {
        // Check if house rule exists
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM house_rules WHERE id = ?)",
            params![house_rule_id],
            |row| row.get(0)
        )?;

        if !exists {
            return Ok(None);
        }

        let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Build dynamic update query
        let mut update_parts = Vec::new();
        let mut params_vec = Vec::new();

        if let Some(title) = &request.title {
            update_parts.push("title = ?");
            params_vec.push(title as &dyn rusqlite::ToSql);
        }
        if let Some(description) = &request.description {
            update_parts.push("description = ?");
            params_vec.push(description as &dyn rusqlite::ToSql);
        }
        if let Some(category) = &request.category {
            update_parts.push("category = ?");
            params_vec.push(category as &dyn rusqlite::ToSql);
        }
        if let Some(is_active) = &request.is_active {
            update_parts.push("is_active = ?");
            params_vec.push(is_active as &dyn rusqlite::ToSql);
        }

        if update_parts.is_empty() {
            // No updates requested, just return the current house rule
            return get_house_rule_by_id_sync(conn, house_rule_id).map(Some);
        }

        update_parts.push("updated_at = ?");
        params_vec.push(&now_str as &dyn rusqlite::ToSql);
        params_vec.push(&house_rule_id as &dyn rusqlite::ToSql);

        let query = format!(
            "UPDATE house_rules SET {} WHERE id = ?",
            update_parts.join(", ")
        );

        conn.execute(&query, params_vec.as_slice())?;

        get_house_rule_by_id_sync(conn, house_rule_id).map(Some)
    })
}

pub async fn delete_house_rule(db: &Database, house_rule_id: HouseRuleId) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute(
            "DELETE FROM house_rules WHERE id = ?",
            params![house_rule_id]
        )?;
        Ok(rows_affected > 0)
    })
}

pub async fn list_house_rules_by_game(db: &Database, game_id: GameId, active_only: bool) -> SqliteResult<Vec<HouseRule>> {
    db.with_connection(|conn| {
        let query = if active_only {
            r#"
            SELECT id, game_id, title, description, category, is_active, created_at, updated_at
            FROM house_rules 
            WHERE game_id = ? AND is_active = TRUE
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT id, game_id, title, description, category, is_active, created_at, updated_at
            FROM house_rules 
            WHERE game_id = ?
            ORDER BY created_at DESC
            "#
        };

        let mut stmt = conn.prepare(query)?;

        let house_rule_iter = stmt.query_map(params![game_id], |row| {
            Ok(HouseRule {
                id: row.get(0)?,
                game_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                is_active: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        })?;

        let house_rules: Result<Vec<HouseRule>, _> = house_rule_iter.collect();
        house_rules
    })
}

// Helper function for synchronous house rule retrieval within transactions
fn get_house_rule_by_id_sync(conn: &rusqlite::Connection, house_rule_id: HouseRuleId) -> SqliteResult<HouseRule> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, game_id, title, description, category, is_active, created_at, updated_at
        FROM house_rules WHERE id = ?
        "#
    )?;

    stmt.query_row(params![house_rule_id], |row| {
        Ok(HouseRule {
            id: row.get(0)?,
            game_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            category: row.get(4)?,
            is_active: row.get(5)?,
            created_at: parse_datetime(row, "created_at")?,
            updated_at: parse_datetime(row, "updated_at")?,
        })
    })
}