use anyhow::{Result, anyhow};
use rig_core::operation::Embedding;
use rig_core::prelude::{DynModel, Embed, EmbeddingsBuilder};

use crate::models::HouseRule;

/// A rulebook text chunk ready for batched embedding via Rig's [`EmbeddingsBuilder`].
#[derive(Debug, Clone, Embed)]
pub struct RuleChunkDocument {
    pub chunk_index: usize,
    #[embed]
    pub text: String,
}

/// A formatted house rule ready for embedding via Rig's [`EmbeddingsBuilder`].
#[derive(Debug, Clone, Embed)]
pub struct HouseRuleDocument {
    #[embed]
    pub content: String,
}

impl HouseRuleDocument {
    pub fn from_house_rule(house_rule: &HouseRule) -> Self {
        let content = format!(
            "House Rule: {}\n{}\n{}",
            house_rule.title,
            house_rule.description,
            house_rule
                .category
                .as_ref()
                .map(|c| format!("Category: {}", c))
                .unwrap_or_default()
        );
        Self { content }
    }
}

/// Service for generating embeddings via Rig's erased [`DynModel`] and [`EmbeddingsBuilder`].
pub struct Embedder {
    model: DynModel<Embedding>,
    model_name: String,
}

impl Embedder {
    pub fn new(model: impl Into<DynModel<Embedding>>, model_name: String) -> Self {
        Self {
            model: model.into(),
            model_name,
        }
    }

    /// Cloned handle to the underlying erased model (cheap Arc clone) for sharing with search tools.
    pub fn model_handle(&self) -> DynModel<Embedding> {
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

    /// Generate embeddings for structured documents using Rig's [`EmbeddingsBuilder`].
    ///
    /// Automatically splits large document sets into provider-sized batches while
    /// preserving document order.
    pub async fn embed_documents<T>(
        &self,
        documents: impl IntoIterator<Item = T>,
    ) -> Result<Vec<(T, Vec<f32>)>>
    where
        T: Embed + Send,
    {
        let docs: Vec<T> = documents.into_iter().collect();
        if docs.is_empty() {
            return Ok(vec![]);
        }

        let embedded = EmbeddingsBuilder::new(self.model.clone())
            .documents(docs)
            .map_err(|e| anyhow!("Failed to prepare documents for embedding: {e}"))?
            .build()
            .await
            .map_err(|e| anyhow!("Failed to create embeddings: {e}"))?;

        embedded
            .into_iter()
            .map(|(doc, vecs)| {
                let mut iter = vecs.into_iter();
                let embedding = iter
                    .next()
                    .ok_or_else(|| anyhow!("Embedding provider returned no vector for document"))?;
                if iter.next().is_some() {
                    return Err(anyhow!(
                        "Embedding provider returned multiple vectors for a single document"
                    ));
                }
                Ok((doc, embedding.vec.into_iter().map(|v| v as f32).collect()))
            })
            .collect()
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
    use rig_core::test_utils::MockEmbeddings;

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
        let model = client.embedding(&config.embedding_model, Some(NOMIC_EMBED_DIMS));
        let service = Embedder::new(model, config.embedding_model.clone());
        assert_eq!(service.model_name(), "nomic-embed-text:latest");
    }

    #[tokio::test]
    async fn test_custom_model() {
        let config = test_config();
        let client = config.build_client().unwrap();
        let model = client.embedding("custom-model", Some(NOMIC_EMBED_DIMS));
        let service = Embedder::new(model, "custom-model".to_string());
        assert_eq!(service.model_name(), "custom-model");
    }

    #[tokio::test]
    async fn test_mock_single_embedding() {
        let service = Embedder::new(MockEmbeddings::model(), "mock-embed".to_string());
        let vec = service.generate_embedding("Movement rules").await.unwrap();
        assert_eq!(vec.len(), 10);
        assert!((vec[1] - 0.1).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_mock_batched_embed_documents_preserves_order() {
        let service = Embedder::new(MockEmbeddings::model(), "mock-embed".to_string());
        // MockEmbeddings caps batches at 5 documents; 12 chunks exercises multi-batching.
        let chunks: Vec<RuleChunkDocument> = (0..12)
            .map(|i| RuleChunkDocument {
                chunk_index: i,
                text: format!("Rule chunk number {i}"),
            })
            .collect();

        let embedded = service.embed_documents(chunks).await.unwrap();
        assert_eq!(embedded.len(), 12);
        for (expected_idx, (doc, vec)) in embedded.into_iter().enumerate() {
            assert_eq!(doc.chunk_index, expected_idx);
            assert_eq!(doc.text, format!("Rule chunk number {expected_idx}"));
            assert_eq!(vec.len(), 10);
        }
    }

    #[tokio::test]
    async fn test_mock_house_rule_document_embedding() {
        let service = Embedder::new(MockEmbeddings::model(), "mock-embed".to_string());
        let rule = HouseRule {
            id: 1,
            game_id: 10,
            title: "Free Parking Jackpot".to_string(),
            description: "Fines go into the center of the board.".to_string(),
            category: Some("Economy".to_string()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let doc = HouseRuleDocument::from_house_rule(&rule);
        assert!(doc.content.contains("Free Parking Jackpot"));
        assert!(doc.content.contains("Category: Economy"));

        let embedded = service.embed_documents([doc]).await.unwrap();
        assert_eq!(embedded.len(), 1);
        assert_eq!(embedded[0].1.len(), 10);
    }

    #[tokio::test]
    async fn test_mock_house_rule_document_without_category() {
        let service = Embedder::new(MockEmbeddings::model(), "mock-embed".to_string());
        let rule = HouseRule {
            id: 2,
            game_id: 10,
            title: "Quick Start".to_string(),
            description: "Deal 5 cards to each player.".to_string(),
            category: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let doc = HouseRuleDocument::from_house_rule(&rule);
        assert!(doc.content.contains("Quick Start"));
        assert!(!doc.content.contains("Category:"));

        let embedded = service.embed_documents([doc]).await.unwrap();
        assert_eq!(embedded.len(), 1);
        assert_eq!(embedded[0].1.len(), 10);
    }

    #[tokio::test]
    async fn test_mock_empty_embed_documents() {
        let service = Embedder::new(MockEmbeddings::model(), "mock-embed".to_string());
        let empty_docs: Vec<RuleChunkDocument> = vec![];
        let embedded = service.embed_documents(empty_docs).await.unwrap();
        assert!(embedded.is_empty());
    }

    // Note: These tests require a running Ollama instance
    #[tokio::test]
    async fn test_generate_single_embedding() {
        let config = test_config();
        let client = config.build_client().unwrap();
        let model = client.embedding(&config.embedding_model, Some(NOMIC_EMBED_DIMS));
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
        let model = client.embedding(&config.embedding_model, Some(NOMIC_EMBED_DIMS));
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

        let result = service.embed_documents(texts).await;

        match result {
            Ok(embedded) => {
                let embeddings: Vec<Vec<f32>> = embedded.into_iter().map(|(_, v)| v).collect();
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
        let model = client.embedding(&config.embedding_model, Some(NOMIC_EMBED_DIMS));
        let service = Embedder::new(model, config.embedding_model.clone());
        let empty_texts: Vec<String> = vec![];

        let result = service.embed_documents(empty_texts).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
