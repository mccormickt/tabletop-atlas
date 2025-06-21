use anyhow::{Result, anyhow};
use chrono::Utc;
use pdf_extract::extract_text;
use serde_json::json;
use std::path::Path;

use crate::db::Database;
use crate::models::GameId;

/// Configuration for text chunking
const CHUNK_SIZE: usize = 1000; // characters per chunk
const CHUNK_OVERLAP: usize = 200; // overlap between chunks

/// PDF processor for extracting text and generating embeddings
pub struct PdfProcessor {
    // In a real application, you'd have an embedding model here
    // For now, we'll simulate embeddings
}

impl PdfProcessor {
    pub fn new() -> Self {
        Self {}
    }

    /// Extract text from a PDF file
    pub async fn extract_text_from_pdf(&self, pdf_path: &Path) -> Result<String> {
        let text = extract_text(pdf_path)
            .map_err(|e| anyhow!("Failed to extract text from PDF: {}", e))?;

        Ok(text)
    }

    /// Split text into chunks for embedding
    pub fn chunk_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();

        if chars.is_empty() {
            return chunks;
        }

        let mut start = 0;

        while start < chars.len() {
            let end = std::cmp::min(start + CHUNK_SIZE, chars.len());
            let chunk: String = chars[start..end].iter().collect();

            // Clean up the chunk - remove excessive whitespace
            let cleaned_chunk = chunk
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            if !cleaned_chunk.trim().is_empty() {
                chunks.push(cleaned_chunk);
            }

            // Move start position with overlap
            start = if end == chars.len() {
                break;
            } else {
                std::cmp::max(start + CHUNK_SIZE - CHUNK_OVERLAP, start + 1)
            };
        }

        chunks
    }

    /// Generate a mock embedding for a text chunk
    /// In a real application, this would call an embedding model API
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // This is a mock implementation that generates a deterministic embedding
        // based on text characteristics. In production, you'd use a real embedding model.

        let mut embedding = vec![0.0f32; 384]; // Common embedding dimension
        let text_bytes = text.as_bytes();

        // Generate pseudo-embedding based on text characteristics
        for (i, &byte) in text_bytes.iter().enumerate() {
            let idx = i % embedding.len();
            embedding[idx] += (byte as f32) / 255.0;
        }

        // Normalize the vector
        let magnitude: f32 = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }

        Ok(embedding)
    }

    /// Process a PDF and return text and chunk data for database storage
    pub async fn process_pdf_for_game(
        &self,
        _game_id: GameId,
        pdf_path: &Path,
        file_name: &str,
    ) -> Result<(String, Vec<ChunkData>)> {
        // Extract text from PDF
        let text = self.extract_text_from_pdf(pdf_path).await?;

        // Chunk the text
        let chunks = self.chunk_text(&text);

        let mut chunk_data = Vec::new();

        // Process each chunk (generate embeddings)
        for (chunk_index, chunk) in chunks.iter().enumerate() {
            // Generate embedding
            let embedding = self.generate_embedding(chunk).await?;

            // Create metadata
            let metadata = json!({
                "file_name": file_name,
                "chunk_size": chunk.len(),
                "total_chunks": chunks.len(),
                "processing_timestamp": Utc::now().to_rfc3339()
            });

            chunk_data.push(ChunkData {
                text: chunk.clone(),
                embedding,
                index: chunk_index,
                metadata: metadata.to_string(),
            });
        }

        Ok((text, chunk_data))
    }

    /// Process a complete PDF workflow - extract, chunk, embed, and store
    pub async fn process_and_store_pdf(
        &self,
        db: &Database,
        game_id: GameId,
        pdf_path: &Path,
        file_name: &str,
    ) -> Result<ProcessingResult> {
        // Process PDF to get text and chunks
        let (text, chunk_data) = self
            .process_pdf_for_game(game_id, pdf_path, file_name)
            .await?;

        // Store in database using consolidated DB function
        let result = crate::db::embeddings::store_pdf_chunks_in_database(
            db,
            game_id,
            pdf_path,
            &text,
            &chunk_data,
        )
        .await
        .map_err(|e| anyhow!("Failed to store PDF chunks in database: {}", e))?;

        Ok(result)
    }

    /// Search for similar chunks using text-based similarity
    /// Note: This is a simplified implementation. In production, you'd use sqlite-vec's
    /// vector search capabilities for better performance
    pub async fn search_similar_chunks(
        &self,
        db: &Database,
        game_id: GameId,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<SimilarChunk>> {
        // Generate embedding for the query (for future vector search)
        let _query_embedding = self.generate_embedding(query_text).await?;

        // Use consolidated database function for search
        let results = crate::db::embeddings::search_pdf_chunks(db, game_id, query_text, limit)
            .await
            .map_err(|e| anyhow!("Failed to search PDF chunks: {}", e))?;

        Ok(results)
    }
}

impl Default for PdfProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of PDF processing
#[derive(Debug)]
pub struct ProcessingResult {
    pub total_text_length: usize,
    pub chunks_processed: u32,
    pub file_path: String,
}

/// Data for a processed text chunk
#[derive(Debug)]
pub struct ChunkData {
    pub text: String,
    pub embedding: Vec<f32>,
    pub index: usize,
    pub metadata: String,
}

/// A chunk similar to a search query
#[derive(Debug)]
pub struct SimilarChunk {
    pub id: i64,
    pub chunk_text: String,
    pub metadata: String,
    pub chunk_index: i32,
    pub similarity_score: f32,
}

/// Validate that a file is a PDF
pub fn validate_pdf_file(file_bytes: &[u8]) -> Result<()> {
    // Check PDF magic number
    if file_bytes.len() < 4 {
        return Err(anyhow!("File too small to be a valid PDF"));
    }

    if &file_bytes[0..4] == b"%PDF" {
        Ok(())
    } else {
        Err(anyhow!("File does not appear to be a valid PDF"))
    }
}

/// Generate a safe filename for storing uploaded PDFs
pub fn generate_pdf_filename(game_id: GameId, original_filename: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let _safe_name = original_filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>();

    format!("game_{}_{}.pdf", game_id, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let processor = PdfProcessor::new();
        let text = "This is a test text that should be chunked. ".repeat(50);
        let chunks = processor.chunk_text(&text);

        assert!(!chunks.is_empty());
        assert!(chunks[0].len() <= CHUNK_SIZE);
    }

    #[test]
    fn test_validate_pdf_file() {
        let pdf_bytes = b"%PDF-1.4 fake pdf content";
        assert!(validate_pdf_file(pdf_bytes).is_ok());

        let not_pdf_bytes = b"This is not a PDF file";
        assert!(validate_pdf_file(not_pdf_bytes).is_err());
    }

    #[test]
    fn test_generate_pdf_filename() {
        let game_id: GameId = 123;
        let original = "My Game Rules.pdf";
        let filename = generate_pdf_filename(game_id, original);

        assert!(filename.starts_with("game_123_"));
        assert!(filename.ends_with(".pdf"));
    }

    #[tokio::test]
    async fn test_generate_embedding() {
        let processor = PdfProcessor::new();
        let embedding = processor.generate_embedding("test text").await.unwrap();

        assert_eq!(embedding.len(), 384);

        // Check that the embedding is normalized
        let magnitude: f32 = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }
}
