use rig_core::DynModel;
use rig_core::operation::Embedding;
use rig_core::vector_store::{
    VectorSearchIdResult, VectorSearchRequest, VectorSearchResult, VectorStoreError,
    VectorStoreIndex,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::db::Database;
use crate::models::SimilaritySearchRequest;

/// No-op filter type to satisfy VectorStoreIndex bounds.
///
/// Game-level filtering (game_id, source_type) is baked into GameRulesIndex at
/// construction time, so external filters are not needed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoFilter;

impl rig_core::vector_store::request::SearchFilter for NoFilter {
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
pub struct GameRulesIndex {
    model: DynModel<Embedding>,
    db: Database,
    game_id: i64,
    include_house_rules: bool,
}

impl GameRulesIndex {
    pub fn new(
        model: DynModel<Embedding>,
        db: Database,
        game_id: i64,
        include_house_rules: bool,
    ) -> Self {
        Self {
            model,
            db,
            game_id,
            include_house_rules,
        }
    }
}

impl VectorStoreIndex for GameRulesIndex {
    type Filter = NoFilter;

    async fn top_n<T: DeserializeOwned + Send>(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<VectorSearchResult<T>>, VectorStoreError> {
        // 1. Embed the query text
        let query_embedding = self
            .model
            .embed_text(req.query())
            .await
            .map_err(VectorStoreError::EmbeddingError)?;

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

        let search_results = search_results.map_err(VectorStoreError::datastore)?;

        // 3. Build VectorSearchResult<T> items
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

            let document: T = serde_json::from_value(doc_json)?;

            results.push(VectorSearchResult {
                score: result.similarity_score as f64,
                id: result.id.to_string(),
                document,
            });
        }

        Ok(results)
    }

    async fn top_n_ids(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<VectorSearchIdResult>, VectorStoreError> {
        let results: Vec<VectorSearchResult<serde_json::Value>> = self.top_n(req).await?;
        Ok(results
            .into_iter()
            .map(|r| VectorSearchIdResult {
                score: r.score,
                id: r.id,
            })
            .collect())
    }
}
