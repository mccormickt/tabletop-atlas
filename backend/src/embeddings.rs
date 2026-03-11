use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use rig::embeddings::EmbeddingModel;

/// Maximum number of texts to embed in a single API call.
/// Keeps request/response sizes manageable for the Ollama server.
const EMBED_BATCH_SIZE: usize = 50;

/// Maximum retry attempts for transient failures.
const MAX_RETRIES: u32 = 3;

/// Service for generating embeddings via the Rig framework.
///
/// Generic over `M` so callers can swap in test doubles. In production,
/// `M = ollama::EmbeddingModel`.
pub struct Embedder<M: EmbeddingModel> {
    model: Arc<M>,
    model_name: String,
}

impl<M: EmbeddingModel> Embedder<M> {
    pub fn new(model: M, model_name: String) -> Self {
        Self {
            model: Arc::new(model),
            model_name,
        }
    }

    /// Shared reference to the underlying model.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Arc-wrapped model for sharing with GameRulesIndex and EmbeddingsBuilder.
    pub fn model_arc(&self) -> Arc<M> {
        self.model.clone()
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Generate an embedding for a single text.
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embedding = self
            .model
            .embed_text(text)
            .await
            .map_err(|e| anyhow!("Failed to create embedding: {e}"))?;

        Ok(embedding.vec.into_iter().map(|v| v as f32).collect())
    }

    /// Embed a batch of texts with retry logic for transient failures.
    async fn embed_batch_with_retry(&self, batch: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut last_err = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = Duration::from_secs(1 << attempt); // 2s, 4s, 8s
                tokio::time::sleep(backoff).await;
            }

            match self.model.embed_texts(batch.clone()).await {
                Ok(embeddings) => {
                    return Ok(embeddings
                        .into_iter()
                        .map(|e| e.vec.into_iter().map(|v| v as f32).collect())
                        .collect());
                }
                Err(e) => {
                    let err_str = e.to_string();
                    let is_transient = err_str.contains("error sending request")
                        || err_str.contains("connection")
                        || err_str.contains("timed out")
                        || err_str.contains("reset by peer");

                    if !is_transient || attempt == MAX_RETRIES {
                        last_err = Some(e);
                        break;
                    }
                    last_err = Some(e);
                }
            }
        }

        Err(anyhow!(
            "Failed to create embeddings after {} retries: {}. \
             Verify that the Ollama server is reachable and the embedding model is available.",
            MAX_RETRIES,
            last_err.unwrap()
        ))
    }

    /// Generate embeddings for multiple texts, processing in batches with retries.
    pub async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

        for (batch_idx, chunk) in texts.chunks(EMBED_BATCH_SIZE).enumerate() {
            let batch_embeddings =
                self.embed_batch_with_retry(chunk.to_vec())
                    .await
                    .map_err(|e| {
                        anyhow!(
                            "Embedding batch {}/{} failed: {e}",
                            batch_idx + 1,
                            texts.len().div_ceil(EMBED_BATCH_SIZE),
                        )
                    })?;

            if batch_embeddings.len() != chunk.len() {
                return Err(anyhow!(
                    "Batch {}: expected {} embeddings, got {}",
                    batch_idx + 1,
                    chunk.len(),
                    batch_embeddings.len()
                ));
            }

            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    /// Test the connection to the embedding service.
    pub async fn test_connection(&self) -> Result<()> {
        self.generate_embedding("test").await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OllamaConfig;
    use rig::prelude::EmbeddingsClient;

    fn test_config() -> OllamaConfig {
        OllamaConfig {
            api_base: "http://localhost:11434".to_string(),
            llm_model: "gpt-oss:latest".to_string(),
            embedding_model: "nomic-embed-text:latest".to_string(),
        }
    }

    const NOMIC_EMBED_DIMS: usize = 768;

    #[tokio::test]
    async fn test_embedding_service_creation() {
        let config = test_config();
        let client = config.build_client().unwrap();
        let model = client.embedding_model_with_ndims(&config.embedding_model, NOMIC_EMBED_DIMS);
        let service = Embedder::new(model, config.embedding_model.clone());
        assert_eq!(service.model_name(), "nomic-embed-text:latest");
    }

    #[tokio::test]
    async fn test_custom_model() {
        let config = test_config();
        let client = config.build_client().unwrap();
        let model = client.embedding_model_with_ndims("custom-model", NOMIC_EMBED_DIMS);
        let service = Embedder::new(model, "custom-model".to_string());
        assert_eq!(service.model_name(), "custom-model");
    }

    // Note: These tests require a running Ollama instance
    #[tokio::test]
    async fn test_generate_single_embedding() {
        let config = test_config();
        let client = config.build_client().unwrap();
        let model = client.embedding_model_with_ndims(&config.embedding_model, NOMIC_EMBED_DIMS);
        let service = Embedder::new(model, config.embedding_model.clone());

        if service.test_connection().await.is_err() {
            println!("Skipping embedding test - Ollama not available");
            return;
        }

        let result = service.generate_embedding("Hello world").await;

        match result {
            Ok(embedding) => {
                assert!(!embedding.is_empty());
                assert!(embedding.len() > 100);
                println!("Generated embedding with {} dimensions", embedding.len());

                let magnitude: f32 = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
                assert!(magnitude > 0.0);
                println!("   Vector magnitude: {:.6}", magnitude);
            }
            Err(e) => {
                panic!("Embedding generation failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_generate_multiple_embeddings() {
        let config = test_config();
        let client = config.build_client().unwrap();
        let model = client.embedding_model_with_ndims(&config.embedding_model, NOMIC_EMBED_DIMS);
        let service = Embedder::new(model, config.embedding_model.clone());

        if service.test_connection().await.is_err() {
            println!("Skipping embedding test - Ollama not available");
            return;
        }

        let texts = vec![
            "Combat rules in board games".to_string(),
            "Movement mechanics explanation".to_string(),
            "Victory conditions and scoring".to_string(),
        ];

        let result = service.generate_embeddings(&texts).await;

        match result {
            Ok(embeddings) => {
                assert_eq!(embeddings.len(), 3);
                for (i, embedding) in embeddings.iter().enumerate() {
                    assert!(!embedding.is_empty());
                    println!("   Embedding {}: {} dimensions", i, embedding.len());
                }

                if embeddings.len() >= 2 {
                    let emb1 = &embeddings[0];
                    let emb2 = &embeddings[1];

                    let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
                    let norm1: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let norm2: f32 = emb2.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let similarity = dot_product / (norm1 * norm2);

                    println!(
                        "   Similarity between 'combat' and 'movement': {:.4}",
                        similarity
                    );
                    assert!(similarity > 0.0);
                }

                println!("Generated {} embeddings successfully", embeddings.len());
            }
            Err(e) => {
                panic!("Multiple embeddings generation failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_empty_input() {
        let config = test_config();
        let client = config.build_client().unwrap();
        let model = client.embedding_model_with_ndims(&config.embedding_model, NOMIC_EMBED_DIMS);
        let service = Embedder::new(model, config.embedding_model.clone());
        let empty_texts: Vec<String> = vec![];

        let result = service.generate_embeddings(&empty_texts).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
