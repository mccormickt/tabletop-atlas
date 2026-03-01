use anyhow::{Result, anyhow};
use rig::client::Nothing;
use rig::embeddings::EmbeddingModel;
use rig::prelude::EmbeddingsClient;
use rig::providers::ollama;

const DEFAULT_API_BASE: &str = "http://localhost:11434";
const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text:latest";
const NOMIC_EMBED_DIMS: usize = 768;

/// Service for generating embeddings using Ollama via the Rig framework.
pub struct Embedder {
    model: ollama::EmbeddingModel,
    model_name: String,
}

impl Default for Embedder {
    fn default() -> Self {
        Self::with_config(DEFAULT_API_BASE, DEFAULT_EMBEDDING_MODEL)
    }
}

impl Embedder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new embedding service with custom Ollama URL and model.
    pub fn with_config(api_base: &str, embedding_model: &str) -> Self {
        let client = ollama::Client::builder()
            .api_key(Nothing)
            .base_url(api_base)
            .build()
            .expect("failed to build Ollama client");

        let model = client.embedding_model_with_ndims(embedding_model, NOMIC_EMBED_DIMS);

        Self {
            model,
            model_name: embedding_model.to_string(),
        }
    }

    /// Generate an embedding for a single text.
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embedding = self
            .model
            .embed_text(text)
            .await
            .map_err(|e| anyhow!("Failed to create embedding: {}", e))?;

        Ok(embedding.vec.into_iter().map(|v| v as f32).collect())
    }

    /// Generate embeddings for multiple texts in a single request.
    pub async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let embeddings = self
            .model
            .embed_texts(texts.to_vec())
            .await
            .map_err(|e| anyhow!("Failed to create embeddings: {}", e))?;

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

    /// Get the embedding model being used.
    pub fn get_model(&self) -> &str {
        &self.model_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_service_creation() {
        let service = Embedder::new();
        assert_eq!(service.get_model(), "nomic-embed-text:latest");
    }

    #[tokio::test]
    async fn test_custom_config() {
        let service = Embedder::with_config("http://localhost:11434", "custom-model");
        assert_eq!(service.get_model(), "custom-model");
    }

    // Note: These tests require a running Ollama instance
    // They will be skipped if Ollama is not available
    #[tokio::test]
    async fn test_generate_single_embedding() {
        let service = Embedder::new();

        // Test connection first
        if service.test_connection().await.is_err() {
            println!("Skipping embedding test - Ollama not available");
            return;
        }

        let result = service.generate_embedding("Hello world").await;

        match result {
            Ok(embedding) => {
                assert!(!embedding.is_empty());
                assert!(embedding.len() > 100); // nomic-embed-text has 768 dimensions
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
        let service = Embedder::new();

        // Test connection first
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
        let service = Embedder::new();
        let empty_texts: Vec<String> = vec![];

        let result = service.generate_embeddings(&empty_texts).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_service_configuration() {
        let custom_service = Embedder::with_config("http://localhost:11434", "custom-model");
        assert_eq!(custom_service.get_model(), "custom-model");
    }
}
