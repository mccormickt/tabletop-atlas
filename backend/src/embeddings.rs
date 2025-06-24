use anyhow::{Result, anyhow};
use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};

const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text:latest";

/// Service for generating embeddings using OpenAI-compatible APIs (like Ollama)
pub struct Embedder {
    client: Client<OpenAIConfig>,
    embedding_model: String,
}

/// Initialize a new embedding service configured for Ollama
impl Default for Embedder {
    fn default() -> Self {
        // Configure for local Ollama instance
        let api_base = "http://localhost:11434/v1";
        let api_key = "ollama"; // Required but ignored by Ollama

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base);

        let client = Client::with_config(config);

        Self {
            client,
            embedding_model: DEFAULT_EMBEDDING_MODEL.to_string(),
        }
    }
}

impl Embedder {
    /// Create a new embedding service configured for Ollama by default
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new embedding service with custom configuration
    pub fn with_config(api_base: &str, api_key: &str, embedding_model: &str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base);

        let client = Client::with_config(config);

        Self {
            client,
            embedding_model: embedding_model.to_string(),
        }
    }

    /// Generate an embedding for a single text
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model)
            .input([text])
            .build()
            .map_err(|e| anyhow!("Failed to build embedding request: {}", e))?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| anyhow!("Failed to create embedding: {}", e))?;

        if response.data.is_empty() {
            return Err(anyhow!("No embedding data returned"));
        }

        Ok(response.data[0].embedding.clone())
    }

    /// Generate embeddings for multiple texts in a single request
    pub async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Convert Vec<String> to Vec<&str> for compatibility
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model)
            .input(text_refs)
            .build()
            .map_err(|e| anyhow!("Failed to build embedding request: {}", e))?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| anyhow!("Failed to create embeddings: {}", e))?;

        if response.data.len() != texts.len() {
            return Err(anyhow!(
                "Expected {} embeddings, got {}",
                texts.len(),
                response.data.len()
            ));
        }

        // Sort by index to ensure correct order
        let mut data = response.data;
        data.sort_by_key(|d| d.index);

        Ok(data.into_iter().map(|d| d.embedding).collect())
    }

    /// Test the connection to the embedding service
    pub async fn test_connection(&self) -> Result<()> {
        self.generate_embedding("test").await?;
        Ok(())
    }

    /// Get the embedding model being used
    pub fn get_model(&self) -> &str {
        &self.embedding_model
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
        let service =
            Embedder::with_config("http://localhost:11434/v1", "test-key", "custom-model");
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
                println!("✅ Generated embedding with {} dimensions", embedding.len());

                // Check if vector is normalized (common for embedding models)
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

                // Test similarity between related concepts
                if embeddings.len() >= 2 {
                    let emb1 = &embeddings[0]; // Combat rules
                    let emb2 = &embeddings[1]; // Movement mechanics

                    // Simple cosine similarity
                    let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
                    let norm1: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let norm2: f32 = emb2.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let similarity = dot_product / (norm1 * norm2);

                    println!(
                        "   Similarity between 'combat' and 'movement': {:.4}",
                        similarity
                    );
                    // Should have some similarity since both are game mechanics
                    assert!(similarity > 0.0);
                }

                println!("✅ Generated {} embeddings successfully", embeddings.len());
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
        let custom_service =
            Embedder::with_config("http://localhost:11434/v1", "test-key", "custom-model");

        assert_eq!(custom_service.get_model(), "custom-model");
    }
}
