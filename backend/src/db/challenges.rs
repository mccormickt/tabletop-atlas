use super::{Database, PaginatedQuery, format_now_for_db, parse_datetime, query_row_optional};
use crate::models::{
    AddParticipantRequest, AssignGameRequest, Challenge, ChallengeGame, ChallengeParticipant,
    ChallengePlay, ChallengePlayWithParticipants, ChallengeStats, ChallengeStatus,
    ChallengeSummary, CreateChallengeRequest, GameType, LeaderboardEntry, PaginatedResponse,
    ParticipantRole, PlayParticipant, RecordPlayRequest, UpdateChallengeRequest, UpdatePlayRequest,
};
use chrono::{NaiveDate, Utc};
use rusqlite::{Result as SqliteResult, Row, params};

fn parse_date(row: &Row, column: &str) -> SqliteResult<Option<NaiveDate>> {
    let date_str: Option<String> = row.get(column)?;
    Ok(date_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()))
}

fn parse_status(status_str: &str) -> ChallengeStatus {
    status_str.parse().unwrap_or(ChallengeStatus::Active)
}

fn parse_game_type(type_str: &str) -> GameType {
    type_str.parse().unwrap_or(GameType::Master)
}

fn parse_role(role_str: &str) -> ParticipantRole {
    role_str.parse().unwrap_or(ParticipantRole::Participant)
}

// Challenge CRUD operations

pub async fn create_challenge(
    db: &Database,
    owner_id: i64,
    request: CreateChallengeRequest,
) -> SqliteResult<Challenge> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        conn.execute(
            r#"
            INSERT INTO challenges (name, description, owner_id, grid_rows, grid_cols, status, start_date, end_date, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, 'active', ?, ?, ?, ?)
            "#,
            params![
                request.name,
                request.description,
                owner_id,
                request.grid_rows.clamp(1, 10),
                request.grid_cols.clamp(1, 10),
                request.start_date.map(|d| d.to_string()),
                request.end_date.map(|d| d.to_string()),
                now_str,
                now_str
            ],
        )?;

        let challenge_id = conn.last_insert_rowid();

        // Add owner as participant
        conn.execute(
            r#"
            INSERT INTO challenge_participants (challenge_id, user_id, role, joined_at)
            VALUES (?, ?, 'owner', ?)
            "#,
            params![challenge_id, owner_id, now_str],
        )?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, owner_id, grid_rows, grid_cols, status, start_date, end_date, created_at, updated_at
            FROM challenges WHERE id = ?
            "#,
        )?;

        stmt.query_row(params![challenge_id], row_to_challenge)
    })
}

pub async fn get_challenge(db: &Database, id: i64) -> SqliteResult<Option<Challenge>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, owner_id, grid_rows, grid_cols, status, start_date, end_date, created_at, updated_at
            FROM challenges WHERE id = ?
            "#,
        )?;

        query_row_optional(stmt.query_row(params![id], row_to_challenge))
    })
}

pub async fn list_user_challenges(
    db: &Database,
    user_id: i64,
    page: u32,
    limit: u32,
) -> SqliteResult<PaginatedResponse<ChallengeSummary>> {
    db.with_connection(|conn| {
        let mut q = PaginatedQuery::new();
        q.filter("cp.user_id = ?", user_id);

        q.execute(
            conn,
            "challenges c JOIN challenge_participants cp ON c.id = cp.challenge_id",
            "c.id, c.name, c.description, c.owner_id, c.grid_rows, c.grid_cols, c.status, c.start_date, c.end_date, c.created_at, (SELECT COUNT(*) FROM challenge_participants WHERE challenge_id = c.id) as participant_count, (SELECT COUNT(*) FROM challenge_plays WHERE challenge_id = c.id) as plays_count",
            "challenges c JOIN challenge_participants cp ON c.id = cp.challenge_id",
            "c.updated_at DESC",
            Some("c.id"),
            page,
            limit,
            |row| {
                let grid_rows: i32 = row.get(4)?;
                let grid_cols: i32 = row.get(5)?;
                let plays_count: i32 = row.get(11)?;
                let total_cells = grid_rows * grid_cols;
                let completion_percentage = if total_cells > 0 {
                    (plays_count as f64 / total_cells as f64) * 100.0
                } else {
                    0.0
                };

                Ok(ChallengeSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    grid_rows,
                    grid_cols,
                    status: parse_status(&row.get::<_, String>(6)?),
                    start_date: parse_date(row, "start_date")?,
                    end_date: parse_date(row, "end_date")?,
                    participant_count: row.get(10)?,
                    completion_percentage,
                    created_at: parse_datetime(row, "created_at")?,
                })
            },
        )
    })
}

pub async fn update_challenge(
    db: &Database,
    id: i64,
    request: UpdateChallengeRequest,
) -> SqliteResult<Option<Challenge>> {
    db.with_transaction(|conn| {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM challenges WHERE id = ?)",
            params![id],
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
        if let Some(status) = request.status {
            update_parts.push("status = ?");
            params_vec.push(Box::new(status.to_string()));
        }
        if let Some(start_date) = request.start_date {
            update_parts.push("start_date = ?");
            params_vec.push(Box::new(start_date.to_string()));
        }
        if let Some(end_date) = request.end_date {
            update_parts.push("end_date = ?");
            params_vec.push(Box::new(end_date.to_string()));
        }

        if !update_parts.is_empty() {
            update_parts.push("updated_at = ?");
            params_vec.push(Box::new(now_str));
            params_vec.push(Box::new(id));

            let query = format!("UPDATE challenges SET {} WHERE id = ?", update_parts.join(", "));
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&query, params_refs.as_slice())?;
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, owner_id, grid_rows, grid_cols, status, start_date, end_date, created_at, updated_at
            FROM challenges WHERE id = ?
            "#,
        )?;

        let challenge = stmt.query_row(params![id], row_to_challenge)?;
        Ok(Some(challenge))
    })
}

pub async fn delete_challenge(db: &Database, id: i64) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows = conn.execute("DELETE FROM challenges WHERE id = ?", params![id])?;
        Ok(rows > 0)
    })
}

// Participant operations

pub async fn add_participant(
    db: &Database,
    challenge_id: i64,
    request: AddParticipantRequest,
) -> SqliteResult<ChallengeParticipant> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        conn.execute(
            r#"
            INSERT INTO challenge_participants (challenge_id, user_id, role, joined_at)
            VALUES (?, ?, 'participant', ?)
            "#,
            params![challenge_id, request.user_id, now_str],
        )?;

        let id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            r#"
            SELECT cp.id, cp.challenge_id, cp.user_id, cp.role, cp.joined_at, u.display_name, u.picture_url
            FROM challenge_participants cp
            LEFT JOIN users u ON cp.user_id = u.id
            WHERE cp.id = ?
            "#,
        )?;

        stmt.query_row(params![id], row_to_participant)
    })
}

pub async fn get_participants(
    db: &Database,
    challenge_id: i64,
) -> SqliteResult<Vec<ChallengeParticipant>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT cp.id, cp.challenge_id, cp.user_id, cp.role, cp.joined_at, u.display_name, u.picture_url
            FROM challenge_participants cp
            LEFT JOIN users u ON cp.user_id = u.id
            WHERE cp.challenge_id = ?
            ORDER BY cp.role DESC, cp.joined_at ASC
            "#,
        )?;

        stmt.query_map(params![challenge_id], row_to_participant)?
            .collect::<Result<Vec<_>, _>>()
    })
}

pub async fn remove_participant(
    db: &Database,
    challenge_id: i64,
    user_id: i64,
) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        // Don't allow removing the owner
        let is_owner: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM challenge_participants WHERE challenge_id = ? AND user_id = ? AND role = 'owner')",
            params![challenge_id, user_id],
            |row| row.get(0),
        )?;

        if is_owner {
            return Ok(false);
        }

        let rows = conn.execute(
            "DELETE FROM challenge_participants WHERE challenge_id = ? AND user_id = ?",
            params![challenge_id, user_id],
        )?;
        Ok(rows > 0)
    })
}

pub async fn is_participant(db: &Database, challenge_id: i64, user_id: i64) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM challenge_participants WHERE challenge_id = ? AND user_id = ?)",
            params![challenge_id, user_id],
            |row| row.get(0),
        )
    })
}

pub async fn is_owner(db: &Database, challenge_id: i64, user_id: i64) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM challenges WHERE id = ? AND owner_id = ?)",
            params![challenge_id, user_id],
            |row| row.get(0),
        )
    })
}

// Game operations

pub async fn assign_game(
    db: &Database,
    challenge_id: i64,
    request: AssignGameRequest,
) -> SqliteResult<ChallengeGame> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        // Replace existing game at this row if any
        conn.execute(
            "DELETE FROM challenge_games WHERE challenge_id = ? AND row_index = ?",
            params![challenge_id, request.row_index],
        )?;

        conn.execute(
            r#"
            INSERT INTO challenge_games (challenge_id, row_index, game_type, game_id, display_name, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            params![
                challenge_id,
                request.row_index,
                request.game_type.to_string(),
                request.game_id,
                request.display_name,
                now_str
            ],
        )?;

        let id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, challenge_id, row_index, game_type, game_id, display_name, created_at
            FROM challenge_games WHERE id = ?
            "#,
        )?;

        stmt.query_row(params![id], row_to_game)
    })
}

pub async fn get_games(db: &Database, challenge_id: i64) -> SqliteResult<Vec<ChallengeGame>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, challenge_id, row_index, game_type, game_id, display_name, created_at
            FROM challenge_games WHERE challenge_id = ?
            ORDER BY row_index ASC
            "#,
        )?;

        stmt.query_map(params![challenge_id], row_to_game)?
            .collect::<Result<Vec<_>, _>>()
    })
}

pub async fn remove_game(db: &Database, challenge_id: i64, game_id: i64) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows = conn.execute(
            "DELETE FROM challenge_games WHERE challenge_id = ? AND id = ?",
            params![challenge_id, game_id],
        )?;
        Ok(rows > 0)
    })
}

/// Check if a game belongs to a challenge
pub async fn game_belongs_to_challenge(
    db: &Database,
    challenge_id: i64,
    game_id: i64,
) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM challenge_games WHERE challenge_id = ? AND id = ?)",
            params![challenge_id, game_id],
            |row| row.get(0),
        )
    })
}

/// Check if a play belongs to a challenge
pub async fn play_belongs_to_challenge(
    db: &Database,
    challenge_id: i64,
    play_id: i64,
) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM challenge_plays WHERE challenge_id = ? AND id = ?)",
            params![challenge_id, play_id],
            |row| row.get(0),
        )
    })
}

/// Check if all user IDs are participants in the challenge
pub async fn validate_play_participants(
    db: &Database,
    challenge_id: i64,
    user_ids: &[i64],
) -> SqliteResult<bool> {
    if user_ids.is_empty() {
        return Ok(true);
    }

    db.with_connection(|conn| {
        for user_id in user_ids {
            let is_participant: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM challenge_participants WHERE challenge_id = ? AND user_id = ?)",
                params![challenge_id, user_id],
                |row| row.get(0),
            )?;
            if !is_participant {
                return Ok(false);
            }
        }
        Ok(true)
    })
}

// Play operations

pub async fn record_play(
    db: &Database,
    challenge_id: i64,
    request: RecordPlayRequest,
) -> SqliteResult<ChallengePlayWithParticipants> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        conn.execute(
            r#"
            INSERT INTO challenge_plays (challenge_id, challenge_game_id, col_index, played_at, notes, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                challenge_id,
                request.challenge_game_id,
                request.col_index,
                request.played_at.to_string(),
                request.notes,
                now_str,
                now_str
            ],
        )?;

        let play_id = conn.last_insert_rowid();

        // Add participants
        for participant in &request.participants {
            conn.execute(
                r#"
                INSERT INTO challenge_play_participants (challenge_play_id, user_id, is_winner, score)
                VALUES (?, ?, ?, ?)
                "#,
                params![play_id, participant.user_id, participant.is_winner, participant.score],
            )?;
        }

        get_play_with_participants_internal(conn, play_id)
    })
}

#[allow(dead_code)]
pub async fn get_play(
    db: &Database,
    play_id: i64,
) -> SqliteResult<Option<ChallengePlayWithParticipants>> {
    db.with_connection(
        |conn| match get_play_with_participants_internal(conn, play_id) {
            Ok(play) => Ok(Some(play)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        },
    )
}

pub async fn update_play(
    db: &Database,
    play_id: i64,
    request: UpdatePlayRequest,
) -> SqliteResult<Option<ChallengePlayWithParticipants>> {
    db.with_transaction(|conn| {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM challenge_plays WHERE id = ?)",
            params![play_id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(None);
        }

        let now_str = format_now_for_db();

        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(played_at) = request.played_at {
            update_parts.push("played_at = ?");
            params_vec.push(Box::new(played_at.to_string()));
        }
        if let Some(notes) = request.notes {
            update_parts.push("notes = ?");
            params_vec.push(Box::new(notes));
        }

        if !update_parts.is_empty() {
            update_parts.push("updated_at = ?");
            params_vec.push(Box::new(now_str));
            params_vec.push(Box::new(play_id));

            let query = format!("UPDATE challenge_plays SET {} WHERE id = ?", update_parts.join(", "));
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&query, params_refs.as_slice())?;
        }

        // Update participants if provided
        if let Some(participants) = request.participants {
            conn.execute(
                "DELETE FROM challenge_play_participants WHERE challenge_play_id = ?",
                params![play_id],
            )?;

            for participant in &participants {
                conn.execute(
                    r#"
                    INSERT INTO challenge_play_participants (challenge_play_id, user_id, is_winner, score)
                    VALUES (?, ?, ?, ?)
                    "#,
                    params![play_id, participant.user_id, participant.is_winner, participant.score],
                )?;
            }
        }

        let play = get_play_with_participants_internal(conn, play_id)?;
        Ok(Some(play))
    })
}

pub async fn delete_play(db: &Database, play_id: i64) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows = conn.execute("DELETE FROM challenge_plays WHERE id = ?", params![play_id])?;
        Ok(rows > 0)
    })
}

pub async fn get_plays(
    db: &Database,
    challenge_id: i64,
) -> SqliteResult<Vec<ChallengePlayWithParticipants>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, challenge_id, challenge_game_id, col_index, played_at, notes, created_at, updated_at
            FROM challenge_plays WHERE challenge_id = ?
            ORDER BY challenge_game_id, col_index
            "#,
        )?;

        let plays: Vec<ChallengePlay> = stmt
            .query_map(params![challenge_id], row_to_play)?
            .collect::<Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for play in plays {
            let participants = get_play_participants_internal(conn, play.id)?;
            result.push(ChallengePlayWithParticipants {
                id: play.id,
                challenge_id: play.challenge_id,
                challenge_game_id: play.challenge_game_id,
                col_index: play.col_index,
                played_at: play.played_at,
                notes: play.notes,
                created_at: play.created_at,
                updated_at: play.updated_at,
                participants,
            });
        }

        Ok(result)
    })
}

// Stats operations

pub async fn get_stats(db: &Database, challenge_id: i64) -> SqliteResult<ChallengeStats> {
    db.with_connection(|conn| {
        // Get challenge dimensions
        let (grid_rows, grid_cols): (i32, i32) = conn.query_row(
            "SELECT grid_rows, grid_cols FROM challenges WHERE id = ?",
            params![challenge_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let total_cells = grid_rows * grid_cols;

        // Get completed cells count
        let completed_cells: i32 = conn.query_row(
            "SELECT COUNT(*) FROM challenge_plays WHERE challenge_id = ?",
            params![challenge_id],
            |row| row.get(0),
        )?;

        let completion_percentage = if total_cells > 0 {
            (completed_cells as f64 / total_cells as f64) * 100.0
        } else {
            0.0
        };

        // Get leaderboard
        let mut stmt = conn.prepare(
            r#"
            SELECT
                cpp.user_id,
                u.display_name,
                u.picture_url,
                SUM(CASE WHEN cpp.is_winner THEN 1 ELSE 0 END) as wins,
                COUNT(*) as total_plays
            FROM challenge_play_participants cpp
            JOIN challenge_plays cp ON cpp.challenge_play_id = cp.id
            LEFT JOIN users u ON cpp.user_id = u.id
            WHERE cp.challenge_id = ?
            GROUP BY cpp.user_id
            ORDER BY wins DESC, total_plays DESC
            "#,
        )?;

        let leaderboard: Vec<LeaderboardEntry> = stmt
            .query_map(params![challenge_id], |row| {
                let wins: i32 = row.get(3)?;
                let total_plays: i32 = row.get(4)?;
                let win_percentage = if total_plays > 0 {
                    (wins as f64 / total_plays as f64) * 100.0
                } else {
                    0.0
                };

                Ok(LeaderboardEntry {
                    user_id: row.get(0)?,
                    display_name: row.get(1)?,
                    picture_url: row.get(2)?,
                    wins,
                    total_plays,
                    win_percentage,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ChallengeStats {
            total_cells,
            completed_cells,
            completion_percentage,
            leaderboard,
        })
    })
}

// Helper functions

fn row_to_challenge(row: &Row) -> SqliteResult<Challenge> {
    Ok(Challenge {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        owner_id: row.get(3)?,
        grid_rows: row.get(4)?,
        grid_cols: row.get(5)?,
        status: parse_status(&row.get::<_, String>(6)?),
        start_date: parse_date(row, "start_date")?,
        end_date: parse_date(row, "end_date")?,
        created_at: parse_datetime(row, "created_at")?,
        updated_at: parse_datetime(row, "updated_at")?,
    })
}

fn row_to_participant(row: &Row) -> SqliteResult<ChallengeParticipant> {
    Ok(ChallengeParticipant {
        id: row.get(0)?,
        challenge_id: row.get(1)?,
        user_id: row.get(2)?,
        role: parse_role(&row.get::<_, String>(3)?),
        joined_at: parse_datetime(row, "joined_at")?,
        display_name: row.get(5)?,
        picture_url: row.get(6)?,
    })
}

fn row_to_game(row: &Row) -> SqliteResult<ChallengeGame> {
    Ok(ChallengeGame {
        id: row.get(0)?,
        challenge_id: row.get(1)?,
        row_index: row.get(2)?,
        game_type: parse_game_type(&row.get::<_, String>(3)?),
        game_id: row.get(4)?,
        display_name: row.get(5)?,
        created_at: parse_datetime(row, "created_at")?,
    })
}

fn row_to_play(row: &Row) -> SqliteResult<ChallengePlay> {
    let played_at_str: String = row.get(4)?;
    let played_at = NaiveDate::parse_from_str(&played_at_str, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());

    Ok(ChallengePlay {
        id: row.get(0)?,
        challenge_id: row.get(1)?,
        challenge_game_id: row.get(2)?,
        col_index: row.get(3)?,
        played_at,
        notes: row.get(5)?,
        created_at: parse_datetime(row, "created_at")?,
        updated_at: parse_datetime(row, "updated_at")?,
    })
}

fn get_play_participants_internal(
    conn: &rusqlite::Connection,
    play_id: i64,
) -> SqliteResult<Vec<PlayParticipant>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT cpp.id, cpp.challenge_play_id, cpp.user_id, cpp.is_winner, cpp.score, u.display_name
        FROM challenge_play_participants cpp
        LEFT JOIN users u ON cpp.user_id = u.id
        WHERE cpp.challenge_play_id = ?
        "#,
    )?;

    stmt.query_map(params![play_id], |row| {
        Ok(PlayParticipant {
            id: row.get(0)?,
            challenge_play_id: row.get(1)?,
            user_id: row.get(2)?,
            is_winner: row.get(3)?,
            score: row.get(4)?,
            display_name: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()
}

fn get_play_with_participants_internal(
    conn: &rusqlite::Connection,
    play_id: i64,
) -> SqliteResult<ChallengePlayWithParticipants> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, challenge_id, challenge_game_id, col_index, played_at, notes, created_at, updated_at
        FROM challenge_plays WHERE id = ?
        "#,
    )?;

    let play: ChallengePlay = stmt.query_row(params![play_id], row_to_play)?;
    let participants = get_play_participants_internal(conn, play_id)?;

    Ok(ChallengePlayWithParticipants {
        id: play.id,
        challenge_id: play.challenge_id,
        challenge_game_id: play.challenge_game_id,
        col_index: play.col_index,
        played_at: play.played_at,
        notes: play.notes,
        created_at: play.created_at,
        updated_at: play.updated_at,
        participants,
    })
}
