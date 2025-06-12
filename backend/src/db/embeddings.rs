use rusqlite::{params, Result as SqliteResult};
use chrono::Utc;
use crate::models::{
    Embedding, EmbeddingId, GameId, HouseRuleId, CreateEmbeddingRequest, 
    EmbeddingSourceType, EmbeddingSearchResult, SimilaritySearchRequest
};
use super::{Database, parse_datetime};

pub async fn create_embedding(db: &Database, request: CreateEmbeddingRequest) -> SqliteResult<Embedding> {
    db.with_transaction(|conn| {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        // Serialize embedding vector as binary data
        let embedding_bytes = serialize_embedding(&request.embedding);

        conn.execute(
            r#"
            INSERT INTO embeddings (
                game_id, chunk_text, embedding, chunk_index, source_type, source_id, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                request.game_id,
                request.chunk_text,
                embedding_bytes,
                request.chunk_index,
                request.source_type.as_str(),
                request.source_id,
                request.metadata,
                now_str
            ]
        )?;

        let embedding_id = conn.last_insert_rowid();

        // Fetch the created embedding
        let mut stmt = conn.prepare(
            r#"
            SELECT id, game_id, chunk_text, embedding, chunk_index, source_type, source_id, metadata, created_at
            FROM embeddings WHERE id = ?
            "#
        )?;

        stmt.query_row(params![embedding_id], |row| {
            let embedding_bytes: Vec<u8> = row.get(3)?;
            let embedding = deserialize_embedding(&embedding_bytes)?;
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

pub async fn get_embeddings_for_game(db: &Database, game_id: GameId, source_type: Option<EmbeddingSourceType>) -> SqliteResult<Vec<Embedding>> {
    db.with_connection(|conn| {
        let (query, params) = if let Some(source_type) = source_type {
            (
                r#"
                SELECT id, game_id, chunk_text, embedding, chunk_index, source_type, source_id, metadata, created_at
                FROM embeddings 
                WHERE game_id = ? AND source_type = ?
                ORDER BY chunk_index ASC
                "#,
                params![game_id, source_type.as_str()]
            )
        } else {
            (
                r#"
                SELECT id, game_id, chunk_text, embedding, chunk_index, source_type, source_id, metadata, created_at
                FROM embeddings 
                WHERE game_id = ?
                ORDER BY source_type ASC, chunk_index ASC
                "#,
                params![game_id]
            )
        };

        let mut stmt = conn.prepare(query)?;

        let embedding_iter = stmt.query_map(params, |row| {
            let embedding_bytes: Vec<u8> = row.get(3)?;
            let embedding = deserialize_embedding(&embedding_bytes)?;
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

        let embeddings: Result<Vec<Embedding>, _> = embedding_iter.collect();
        embeddings
    })
}

pub async fn similarity_search(db: &Database, request: SimilaritySearchRequest) -> SqliteResult<Vec<EmbeddingSearchResult>> {
    db.with_connection(|conn| {
        // Get all embeddings for the game
        let mut stmt = conn.prepare(
            r#"
            SELECT id, chunk_text, embedding, source_type, source_id, metadata
            FROM embeddings 
            WHERE game_id = ?
            "#
        )?;

        let embedding_iter = stmt.query_map(params![request.game_id], |row| {
            let embedding_bytes: Vec<u8> = row.get(2)?;
            let embedding = deserialize_embedding(&embedding_bytes)?;
            let source_type_str: String = row.get(3)?;
            let source_type = EmbeddingSourceType::from_str(&source_type_str)
                .unwrap_or(EmbeddingSourceType::RulesPdf);

            Ok((
                row.get::<_, i64>(0)?, // id
                row.get::<_, String>(1)?, // chunk_text
                embedding,
                source_type,
                row.get::<_, Option<i64>>(4)?, // source_id
                row.get::<_, Option<String>>(5)?, // metadata
            ))
        })?;

        let mut results = Vec::new();
        
        for item in embedding_iter {
            let (id, chunk_text, embedding, source_type, source_id, metadata) = item?;
            
            // Calculate cosine similarity
            let similarity = cosine_similarity(&request.query_embedding, &embedding);
            
            if similarity >= request.similarity_threshold {
                results.push(EmbeddingSearchResult {
                    id,
                    chunk_text,
                    similarity_score: similarity,
                    source_type,
                    source_id,
                    metadata,
                });
            }
        }

        // Sort by similarity score (highest first) and limit results
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(request.limit as usize);

        Ok(results)
    })
}

pub async fn delete_embeddings_for_game(db: &Database, game_id: GameId, source_type: Option<EmbeddingSourceType>) -> SqliteResult<u32> {
    db.with_connection(|conn| {
        let (query, params) = if let Some(source_type) = source_type {
            (
                "DELETE FROM embeddings WHERE game_id = ? AND source_type = ?",
                params![game_id, source_type.as_str()]
            )
        } else {
            (
                "DELETE FROM embeddings WHERE game_id = ?",
                params![game_id]
            )
        };

        let rows_affected = conn.execute(query, params)?;
        Ok(rows_affected as u32)
    })
}

pub async fn delete_embeddings_for_house_rule(db: &Database, house_rule_id: HouseRuleId) -> SqliteResult<u32> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute(
            "DELETE FROM embeddings WHERE source_type = 'house_rule' AND source_id = ?",
            params![house_rule_id]
        )?;
        Ok(rows_affected as u32)
    })
}

pub async fn get_embedding_by_id(db: &Database, embedding_id: EmbeddingId) -> SqliteResult<Option<Embedding>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, game_id, chunk_text, embedding, chunk_index, source_type, source_id, metadata, created_at
            FROM embeddings WHERE id = ?
            "#
        )?;

        let result = stmt.query_row(params![embedding_id], |row| {
            let embedding_bytes: Vec<u8> = row.get(3)?;
            let embedding = deserialize_embedding(&embedding_bytes)?;
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

// Helper functions for embedding serialization
fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    // Simple serialization: convert f32 array to bytes
    // In production, you might want to use a more efficient format
    let mut bytes = Vec::with_capacity(embedding.len() * 4);
    for &value in embedding {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn deserialize_embedding(bytes: &[u8]) -> SqliteResult<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        return Err(rusqlite::Error::InvalidColumnType(
            0,
            "embedding".to_string(),
            rusqlite::types::Type::Blob,
        ));
    }

    let mut embedding = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        embedding.push(value);
    }
    Ok(embedding)
}

// Helper function for cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

// Batch operations for efficiency
pub async fn create_embeddings_batch(db: &Database, requests: Vec<CreateEmbeddingRequest>) -> SqliteResult<Vec<EmbeddingId>> {
    db.with_transaction(|conn| {
        let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut embedding_ids = Vec::new();

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO embeddings (
                game_id, chunk_text, embedding, chunk_index, source_type, source_id, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )?;

        for request in requests {
            let embedding_bytes = serialize_embedding(&request.embedding);
            
            stmt.execute(params![
                request.game_id,
                request.chunk_text,
                embedding_bytes,
                request.chunk_index,
                request.source_type.as_str(),
                request.source_id,
                request.metadata,
                now_str
            ])?;

            embedding_ids.push(conn.last_insert_rowid());
        }

        Ok(embedding_ids)
    })
}