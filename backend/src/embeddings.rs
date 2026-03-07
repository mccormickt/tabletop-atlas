use std::sync::Arc;

use anyhow::{Result, anyhow};
use rig::embeddings::EmbeddingModel;

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

    /// Generate embeddings for multiple texts in a single call.
    pub async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let embeddings = self
            .model
            .embed_texts(texts.to_vec())
            .await
            .map_err(|e| anyhow!("Failed to create embeddings: {e}"))?;

        if embeddings.len() != texts.len() {
            return Err(anyhow!(
                "Expected {} embeddings, got {}",
                texts.len(),
                embeddings.len()
            ));
        }

        Ok(embeddings
            .into_iter()
            .map(|e| e.vec.into_iter().map(|v| v as f32).collect())
            .collect())
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
