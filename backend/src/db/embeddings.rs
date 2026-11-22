use rusqlite::{Result as SqliteResult, params};

use crate::models::{
    CreateEmbeddingRequest, Embedding, EmbeddingSearchResult, EmbeddingSourceType,
    SimilaritySearchRequest,
};

use super::{Database, format_now_for_db, parse_datetime};

/// Type alias for embedding metadata from database queries
type EmbeddingMetadataRow = (i64, String, String, Option<i64>, Option<String>);

pub async fn create_embedding(
    db: &Database,
    request: CreateEmbeddingRequest,
) -> SqliteResult<Embedding> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

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
        let metadata_results: Vec<EmbeddingMetadataRow> = meta_stmt
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
                    let source_type = EmbeddingSourceType::from_str(source_type_str)
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

/// Similarity search with optional house rule filtering
pub async fn similarity_search_filtered(
    db: &Database,
    request: SimilaritySearchRequest,
    include_house_rules: bool,
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

        // Query 2: Get metadata for the vector results, filtered by game_id and optionally by source_type
        let source_filter = if include_house_rules {
            "" // Include all source types
        } else {
            " AND source_type = 'rules_pdf'" // Exclude house rules
        };

        let metadata_query = format!(
            r#"
            SELECT id, chunk_text, source_type, source_id, metadata
            FROM embeddings
            WHERE id IN ({}) AND game_id = ?{}
            ORDER BY
                CASE id {} END
            "#,
            placeholders,
            source_filter,
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
        let metadata_results: Vec<EmbeddingMetadataRow> = meta_stmt
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
                    let source_type = EmbeddingSourceType::from_str(source_type_str)
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

/// Common English filler words excluded from keyword search when more selective terms exist.
const KEYWORD_STOPWORDS: &[&str] = &[
    "a", "about", "all", "an", "and", "any", "are", "as", "at", "be", "but", "by", "can", "do",
    "does", "for", "from", "get", "had", "has", "have", "how", "if", "in", "into", "is", "it",
    "its", "may", "my", "no", "not", "of", "on", "or", "our", "out", "so", "than", "that", "the",
    "their", "them", "then", "there", "these", "they", "this", "to", "too", "up", "use", "was",
    "we", "were", "what", "when", "where", "which", "who", "why", "will", "with", "would", "you",
    "your",
];

/// Case-insensitive keyword search over stored rulebook (and optional house rule) chunks.
///
/// Splits the query into whitespace-separated terms (deduplicated, up to 8 terms, ignoring
/// stopwords when selective terms exist) and matches chunks containing any term, ranking
/// chunks that match more terms higher.
pub async fn keyword_search_filtered(
    db: &Database,
    game_id: i64,
    keywords: &str,
    include_house_rules: bool,
    limit: u32,
) -> SqliteResult<Vec<EmbeddingSearchResult>> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let mut all_tokens: Vec<String> = Vec::new();
    let mut selective_tokens: Vec<String> = Vec::new();
    for raw in keywords.split_whitespace() {
        let cleaned = raw
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        let is_valid = !cleaned.is_empty()
            && cleaned.chars().any(char::is_alphanumeric)
            && (cleaned.chars().count() >= 2 || cleaned.chars().all(|c| c.is_ascii_digit()));
        if !is_valid {
            continue;
        }
        if !all_tokens.contains(&cleaned) {
            if !KEYWORD_STOPWORDS.contains(&cleaned.as_str()) && selective_tokens.len() < 8 {
                selective_tokens.push(cleaned.clone());
            }
            all_tokens.push(cleaned);
        }
    }

    let terms = if !selective_tokens.is_empty() {
        selective_tokens
    } else {
        all_tokens.truncate(8);
        all_tokens
    };

    if terms.is_empty() {
        return Ok(Vec::new());
    }

    db.with_connection(|conn| {
        let source_filter = if include_house_rules {
            ""
        } else {
            " AND source_type = 'rules_pdf'"
        };

        let score_expr = terms
            .iter()
            .map(|_| "CASE WHEN LOWER(chunk_text) LIKE ? ESCAPE '\\' THEN 1 ELSE 0 END")
            .collect::<Vec<_>>()
            .join(" + ");

        let query = format!(
            r#"
            SELECT id, chunk_text, source_type, source_id, metadata, match_count
            FROM (
                SELECT id, chunk_text, source_type, source_id, metadata, chunk_index,
                       ({score_expr}) AS match_count
                FROM embeddings
                WHERE game_id = ?{source_filter}
            )
            WHERE match_count > 0
            ORDER BY match_count DESC, chunk_index ASC
            LIMIT ?
            "#
        );

        let escaped_patterns: Vec<String> = terms
            .iter()
            .map(|t| {
                let escaped = t
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_");
                format!("%{escaped}%")
            })
            .collect();

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> =
            Vec::with_capacity(escaped_patterns.len() + 2);
        for pat in escaped_patterns {
            params_vec.push(Box::new(pat));
        }
        params_vec.push(Box::new(game_id));
        params_vec.push(Box::new(limit));

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let term_count = terms.len() as f32;
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                let id: i64 = row.get(0)?;
                let chunk_text: String = row.get(1)?;
                let source_type_str: String = row.get(2)?;
                let source_id: Option<i64> = row.get(3)?;
                let metadata: Option<String> = row.get(4)?;
                let match_count: i64 = row.get(5)?;
                let source_type = EmbeddingSourceType::from_str(&source_type_str)
                    .unwrap_or(EmbeddingSourceType::RulesPdf);
                let relevance_score = (match_count as f32 / term_count).min(1.0);

                Ok(EmbeddingSearchResult {
                    id,
                    chunk_text,
                    similarity_score: relevance_score,
                    source_type,
                    source_id,
                    metadata,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    })
}

pub async fn delete_embeddings_for_game(
    db: &Database,
    game_id: i64,
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
    house_rule_id: i64,
) -> SqliteResult<u32> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute(
            "DELETE FROM embeddings WHERE source_type = 'house_rule' AND source_id = ?",
            params![house_rule_id],
        )?;
        Ok(rows_affected as u32)
    })
}

// Batch operations for efficiency
pub async fn create_embeddings_batch(
    db: &Database,
    requests: Vec<CreateEmbeddingRequest>,
) -> SqliteResult<Vec<i64>> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_keyword_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE embeddings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                chunk_text TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                source_type TEXT NOT NULL DEFAULT 'rules_pdf',
                source_id INTEGER,
                metadata TEXT,
                created_at TEXT NOT NULL
            );
            INSERT INTO embeddings (game_id, chunk_text, chunk_index, source_type, created_at) VALUES
                (1, 'Rolling a 7 activates the Robber and forces players to discard half.', 0, 'rules_pdf', '2026-01-01T00:00:00Z'),
                (1, 'The Robber blocks resource production on the hex it occupies.', 1, 'rules_pdf', '2026-01-01T00:00:00Z'),
                (1, 'Special discount 50%off and max_hand_size and path\to\rule.', 2, 'rules_pdf', '2026-01-01T00:00:00Z'),
                (1, 'Friendly Robber house rule protects players with 2 VP.', 3, 'house_rule', '2026-01-01T00:00:00Z'),
                (2, 'Other game Robber discard rule should never leak.', 0, 'rules_pdf', '2026-01-01T00:00:00Z');
            "#,
        )
        .unwrap();
        Database::new(conn)
    }

    #[tokio::test]
    async fn test_keyword_search_ranking_and_game_isolation() {
        let db = setup_keyword_db();
        let results = keyword_search_filtered(&db, 1, "Robber discard", false, 5)
            .await
            .unwrap();
        // Only game 1 rules_pdf chunks match; chunk 0 matches both terms so ranks first.
        assert_eq!(results.len(), 2);
        assert!(results[0].chunk_text.contains("discard half"));
        assert!((results[0].similarity_score - 1.0).abs() < f32::EPSILON);
        assert!(results[1].chunk_text.contains("blocks resource production"));
        assert!((results[1].similarity_score - 0.5).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_keyword_search_house_rules_toggle() {
        let db = setup_keyword_db();
        let without_hr = keyword_search_filtered(&db, 1, "Friendly", false, 5)
            .await
            .unwrap();
        assert!(without_hr.is_empty());

        let with_hr = keyword_search_filtered(&db, 1, "Friendly", true, 5)
            .await
            .unwrap();
        assert_eq!(with_hr.len(), 1);
        assert_eq!(with_hr[0].source_type, EmbeddingSourceType::HouseRule);
    }

    #[tokio::test]
    async fn test_keyword_search_single_digit_and_stopwords() {
        let db = setup_keyword_db();
        // Single digit "7" must not be dropped; stopwords like "what", "the" must not dilute matches
        let results = keyword_search_filtered(&db, 1, "what happens on the 7?", false, 5)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].chunk_text.contains("Rolling a 7"));
    }

    #[tokio::test]
    async fn test_keyword_search_escapes_like_wildcards_and_rejects_punctuation() {
        let db = setup_keyword_db();

        // Pure punctuation / empty / limit 0 return empty
        assert!(
            keyword_search_filtered(&db, 1, "--- ''' !!!", true, 5)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            keyword_search_filtered(&db, 1, "Robber", true, 0)
                .await
                .unwrap()
                .is_empty()
        );

        // Escaped `%`, `_`, `\` match literally and do not act as SQL LIKE wildcards
        let pct = keyword_search_filtered(&db, 1, "50%off", false, 5)
            .await
            .unwrap();
        assert_eq!(pct.len(), 1);
        assert!(pct[0].chunk_text.contains("50%off"));

        let underscore = keyword_search_filtered(&db, 1, "max_hand_size", false, 5)
            .await
            .unwrap();
        assert_eq!(underscore.len(), 1);

        let backslash = keyword_search_filtered(&db, 1, "path\\to\\rule", false, 5)
            .await
            .unwrap();
        assert_eq!(backslash.len(), 1);

        // An underscore pattern that does NOT literally exist must not wildcard-match "Robber"
        let non_match = keyword_search_filtered(&db, 1, "r_bber", false, 5)
            .await
            .unwrap();
        assert!(non_match.is_empty());
    }
}
