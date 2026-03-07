use std::sync::Arc;

use rig::embeddings::EmbeddingModel;
use rig::vector_store::VectorStoreError;
use rig::vector_store::VectorStoreIndex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::db::Database;
use crate::models::{GameId, SimilaritySearchRequest};

/// No-op filter type to satisfy VectorStoreIndex + VectorStoreIndexDyn bounds.
///
/// Game-level filtering (game_id, source_type) is baked into GameRulesIndex at
/// construction time, so external filters are not needed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoFilter;

impl rig::vector_store::request::SearchFilter for NoFilter {
    type Value = serde_json::Value;

    fn eq(_key: impl AsRef<str>, _value: Self::Value) -> Self {
        Self
    }
    fn gt(_key: impl AsRef<str>, _value: Self::Value) -> Self {
        Self
    }
    fn lt(_key: impl AsRef<str>, _value: Self::Value) -> Self {
        Self
    }
    fn and(self, _rhs: Self) -> Self {
        Self
    }
    fn or(self, _rhs: Self) -> Self {
        Self
    }
}

/// Per-request vector store index that searches the existing embeddings/vec_embeddings
/// tables for a specific game, optionally including house rules.
///
/// Implements `VectorStoreIndex` so it can be passed to `dynamic_context()` on
/// the rig agent builder.
pub struct GameRulesIndex<M: EmbeddingModel> {
    model: Arc<M>,
    db: Database,
    game_id: GameId,
    include_house_rules: bool,
}

impl<M: EmbeddingModel> GameRulesIndex<M> {
    pub fn new(model: Arc<M>, db: Database, game_id: GameId, include_house_rules: bool) -> Self {
        Self {
            model,
            db,
            game_id,
            include_house_rules,
        }
    }
}

impl<M: EmbeddingModel + Send + Sync + 'static> VectorStoreIndex for GameRulesIndex<M> {
    type Filter = NoFilter;

    async fn top_n<T: DeserializeOwned + Send>(
        &self,
        req: rig::vector_store::VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String, T)>, VectorStoreError> {
        // 1. Embed the query text
        let query_embedding = self
            .model
            .embed_text(req.query())
            .await
            .map_err(|e| VectorStoreError::DatastoreError(Box::new(e)))?;

        let embedding_vec: Vec<f32> = query_embedding.vec.into_iter().map(|v| v as f32).collect();

        // 2. Search our sqlite-vec tables
        let similarity_request = SimilaritySearchRequest {
            game_id: self.game_id,
            query_embedding: embedding_vec,
            similarity_threshold: 0.0,
            limit: req.samples() as u32,
        };

        let search_results = if self.include_house_rules {
            crate::db::embeddings::similarity_search(&self.db, similarity_request).await
        } else {
            crate::db::embeddings::similarity_search_filtered(&self.db, similarity_request, false)
                .await
        };

        let search_results =
            search_results.map_err(|e| VectorStoreError::DatastoreError(Box::new(e)))?;

        // 3. Build (score, id, T) tuples
        let mut results = Vec::new();
        for result in search_results {
            let source_label = match result.source_type {
                crate::models::EmbeddingSourceType::HouseRule => "House Rule",
                crate::models::EmbeddingSourceType::RulesPdf => "Official Rule",
            };

            let doc_json = serde_json::json!({
                "source": source_label,
                "text": result.chunk_text,
            });

            let doc: T = serde_json::from_value(doc_json)
                .map_err(|e| VectorStoreError::DatastoreError(Box::new(e)))?;

            results.push((result.similarity_score as f64, result.id.to_string(), doc));
        }

        Ok(results)
    }

    async fn top_n_ids(
        &self,
        req: rig::vector_store::VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String)>, VectorStoreError> {
        let results: Vec<(f64, String, serde_json::Value)> = self.top_n(req).await?;
        Ok(results
            .into_iter()
            .map(|(score, id, _)| (score, id))
            .collect())
    }
}
