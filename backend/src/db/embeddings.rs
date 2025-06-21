use chrono::Utc;
use rusqlite::{Result as SqliteResult, params};
use serde_json;

use crate::models::{
    CreateEmbeddingRequest, Embedding, EmbeddingId, EmbeddingSearchResult, EmbeddingSourceType,
    GameId, HouseRuleId, SimilaritySearchRequest,
};

use super::{Database, parse_datetime};

pub async fn create_embedding(
    db: &Database,
    request: CreateEmbeddingRequest,
) -> SqliteResult<Embedding> {
    db.with_transaction(|conn| {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        // Convert embedding vector to JSON string for sqlite-vec
        let embedding_json = serde_json::to_string(&request.embedding)
            .map_err(|_| rusqlite::Error::ToSqlConversionFailure(Box::new(std::fmt::Error)))?;

        // Insert into main embeddings table
        conn.execute(
            r#"
            INSERT INTO embeddings (
                game_id, chunk_text, chunk_index, source_type, source_id, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                request.game_id,
                request.chunk_text,
                request.chunk_index,
                request.source_type.as_str(),
                request.source_id,
                request.metadata,
                now_str
            ]
        )?;

        let embedding_id = conn.last_insert_rowid();

        // Insert into vector embeddings table using virtual table syntax
        conn.execute(
            "INSERT INTO vec_embeddings (rowid, embedding_vector) VALUES (?, ?)",
            params![embedding_id, embedding_json]
        )?;

        // Fetch the created embedding
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.game_id, e.chunk_text, v.embedding_vector, e.chunk_index, e.source_type, e.source_id, e.metadata, e.created_at
            FROM embeddings e
            JOIN vec_embeddings v ON e.id = v.rowid
            WHERE e.id = ?
            "#
        )?;

        stmt.query_row(params![embedding_id], |row| {
            let embedding_json: String = row.get(3)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "embedding_vector".to_string(), rusqlite::types::Type::Text))?;
            let source_type_str: String = row.get(5)?;
            let source_type = EmbeddingSourceType::from_str(&source_type_str)
                .unwrap_or(EmbeddingSourceType::RulesPdf);

            Ok(Embedding {
                id: row.get(0)?,
                game_id: row.get(1)?,
                chunk_text: row.get(2)?,
                embedding,
                chunk_index: row.get(4)?,
                source_type,
                source_id: row.get(6)?,
                metadata: row.get(7)?,
                created_at: parse_datetime(row, "created_at")?,
            })
        })
    })
}

pub async fn get_embeddings_for_game(
    db: &Database,
    game_id: GameId,
    source_type: Option<EmbeddingSourceType>,
) -> SqliteResult<Vec<Embedding>> {
    db.with_connection(|conn| {
        let (query, params) = if let Some(source_type) = source_type {
            (
                r#"
                SELECT e.id, e.game_id, e.chunk_text, v.embedding_vector, e.chunk_index, e.source_type, e.source_id, e.metadata, e.created_at
                FROM embeddings e
                JOIN vec_embeddings v ON e.id = v.rowid
                WHERE e.game_id = ? AND e.source_type = ?
                ORDER BY e.chunk_index ASC
                "#,
                params![game_id, source_type.as_str()]
            )
        } else {
            (
                r#"
                SELECT e.id, e.game_id, e.chunk_text, v.embedding_vector, e.chunk_index, e.source_type, e.source_id, e.metadata, e.created_at
                FROM embeddings e
                JOIN vec_embeddings v ON e.id = v.rowid
                WHERE e.game_id = ?
                ORDER BY e.source_type ASC, e.chunk_index ASC
                "#,
                params![game_id]
            )
        };

        let mut stmt = conn.prepare(query)?;

        let embedding_iter = stmt.query_map(params, |row| {
            let embedding_json: String = row.get(3)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "embedding_vector".to_string(), rusqlite::types::Type::Text))?;
            let source_type_str: String = row.get(5)?;
            let source_type = EmbeddingSourceType::from_str(&source_type_str)
                .unwrap_or(EmbeddingSourceType::RulesPdf);

            Ok(Embedding {
                id: row.get(0)?,
                game_id: row.get(1)?,
                chunk_text: row.get(2)?,
                embedding,
                chunk_index: row.get(4)?,
                source_type,
                source_id: row.get(6)?,
                metadata: row.get(7)?,
                created_at: parse_datetime(row, "created_at")?,
            })
        })?;

        embedding_iter.collect()
    })
}

pub async fn similarity_search(
    db: &Database,
    request: SimilaritySearchRequest,
) -> SqliteResult<Vec<EmbeddingSearchResult>> {
    db.with_connection(|conn| {
        // Convert query embedding to JSON for sqlite-vec KNN search
        let query_json = serde_json::to_string(&request.query_embedding)
            .map_err(|_| rusqlite::Error::ToSqlConversionFailure(Box::new(std::fmt::Error)))?;

        // Query 1: Get vector search results from sqlite-vec (no JOINs, no additional filtering)
        let search_limit = std::cmp::max(request.limit * 3, 50); // Get more to allow for filtering
        let mut vec_stmt = conn.prepare(
            r#"
            SELECT rowid, distance
            FROM vec_embeddings
            WHERE embedding_vector MATCH ?1
            ORDER BY distance
            LIMIT ?2
            "#,
        )?;

        let vec_results: Vec<(i64, f32)> = vec_stmt
            .query_map(params![query_json, search_limit], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if vec_results.is_empty() {
            return Ok(Vec::new());
        }

        // Build placeholders for IN clause
        let placeholders: String = vec_results
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");

        // Query 2: Get metadata for the vector results, filtered by game_id
        let metadata_query = format!(
            r#"
            SELECT id, chunk_text, source_type, source_id, metadata
            FROM embeddings
            WHERE id IN ({}) AND game_id = ?
            ORDER BY
                CASE id {} END
            "#,
            placeholders,
            vec_results
                .iter()
                .enumerate()
                .map(|(i, (rowid, _))| format!("WHEN {} THEN {}", rowid, i))
                .collect::<Vec<_>>()
                .join(" ")
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        for (rowid, _) in &vec_results {
            params.push(Box::new(*rowid));
        }
        params.push(Box::new(request.game_id));

        let mut meta_stmt = conn.prepare(&metadata_query)?;
        let metadata_results: Vec<(i64, String, String, Option<i64>, Option<String>)> = meta_stmt
            .query_map(
                params
                    .iter()
                    .map(|p| p.as_ref())
                    .collect::<Vec<_>>()
                    .as_slice(),
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        // Combine results, maintaining distance order and applying similarity threshold
        let mut results = Vec::new();
        for (rowid, distance) in vec_results {
            if let Some((id, chunk_text, source_type_str, source_id, metadata)) = metadata_results
                .iter()
                .find(|(meta_id, _, _, _, _)| *meta_id == rowid)
            {
                let similarity_score = 1.0 - distance as f64;

                // Apply similarity threshold
                if similarity_score >= request.similarity_threshold as f64 {
                    let source_type = EmbeddingSourceType::from_str(&source_type_str)
                        .unwrap_or(EmbeddingSourceType::RulesPdf);

                    results.push(EmbeddingSearchResult {
                        id: *id,
                        chunk_text: chunk_text.clone(),
                        similarity_score: similarity_score as f32,
                        source_type,
                        source_id: *source_id,
                        metadata: metadata.clone(),
                    });

                    // Stop when we have enough results
                    if results.len() >= request.limit as usize {
                        break;
                    }
                }
            }
        }

        Ok(results)
    })
}

pub async fn delete_embeddings_for_game(
    db: &Database,
    game_id: GameId,
    source_type: Option<EmbeddingSourceType>,
) -> SqliteResult<u32> {
    db.with_connection(|conn| {
        let (query, params) = if let Some(source_type) = source_type {
            (
                "DELETE FROM embeddings WHERE game_id = ? AND source_type = ?",
                params![game_id, source_type.as_str()],
            )
        } else {
            ("DELETE FROM embeddings WHERE game_id = ?", params![game_id])
        };

        let rows_affected = conn.execute(query, params)?;
        Ok(rows_affected as u32)
    })
}

pub async fn delete_embeddings_for_house_rule(
    db: &Database,
    house_rule_id: HouseRuleId,
) -> SqliteResult<u32> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute(
            "DELETE FROM embeddings WHERE source_type = 'house_rule' AND source_id = ?",
            params![house_rule_id],
        )?;
        Ok(rows_affected as u32)
    })
}

pub async fn get_embedding_by_id(
    db: &Database,
    embedding_id: EmbeddingId,
) -> SqliteResult<Option<Embedding>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.game_id, e.chunk_text, v.embedding_vector, e.chunk_index, e.source_type, e.source_id, e.metadata, e.created_at
            FROM embeddings e
            JOIN vec_embeddings v ON e.id = v.rowid
            WHERE e.id = ?
            "#
        )?;

        let result = stmt.query_row(params![embedding_id], |row| {
            let embedding_json: String = row.get(3)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "embedding_vector".to_string(), rusqlite::types::Type::Text))?;
            let source_type_str: String = row.get(5)?;
            let source_type = EmbeddingSourceType::from_str(&source_type_str)
                .unwrap_or(EmbeddingSourceType::RulesPdf);

            Ok(Embedding {
                id: row.get(0)?,
                game_id: row.get(1)?,
                chunk_text: row.get(2)?,
                embedding,
                chunk_index: row.get(4)?,
                source_type,
                source_id: row.get(6)?,
                metadata: row.get(7)?,
                created_at: parse_datetime(row, "created_at")?,
            })
        });

        match result {
            Ok(embedding) => Ok(Some(embedding)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    })
}

// Batch operations for efficiency
pub async fn create_embeddings_batch(
    db: &Database,
    requests: Vec<CreateEmbeddingRequest>,
) -> SqliteResult<Vec<EmbeddingId>> {
    db.with_transaction(|conn| {
        let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut embedding_ids = Vec::new();

        let mut embeddings_stmt = conn.prepare(
            r#"
            INSERT INTO embeddings (
                game_id, chunk_text, chunk_index, source_type, source_id, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;

        let mut vec_stmt =
            conn.prepare("INSERT INTO vec_embeddings (rowid, embedding_vector) VALUES (?, ?)")?;

        for request in requests {
            // Convert embedding to JSON for sqlite-vec
            let embedding_json = serde_json::to_string(&request.embedding)
                .map_err(|_| rusqlite::Error::ToSqlConversionFailure(Box::new(std::fmt::Error)))?;

            // Insert into embeddings table
            embeddings_stmt.execute(params![
                request.game_id,
                request.chunk_text,
                request.chunk_index,
                request.source_type.as_str(),
                request.source_id,
                request.metadata,
                now_str
            ])?;

            let embedding_id = conn.last_insert_rowid();

            // Insert into vec_embeddings table
            vec_stmt.execute(params![embedding_id, embedding_json])?;

            embedding_ids.push(embedding_id);
        }

        Ok(embedding_ids)
    })
}
