use anyhow::{Result, anyhow};
use pdf_extract::extract_text;
use std::path::Path;

/// Configuration for text chunking
const CHUNK_SIZE: usize = 1000; // characters per chunk
const CHUNK_OVERLAP: usize = 200; // overlap between chunks

/// Simple PDF service that only handles PDF text extraction and chunking
/// Database and embedding operations are handled separately
pub struct Processor;

impl Processor {
    pub fn new() -> Self {
        Self
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

    /// Process a PDF file and return extracted text and chunks
    /// This is a pure processing function that doesn't touch the database or embeddings
    pub async fn process_pdf(&self, pdf_path: &Path) -> Result<ProcessedPdf> {
        // Extract text from PDF
        let text = self.extract_text_from_pdf(pdf_path).await?;

        // Chunk the text
        let chunks = self.chunk_text(&text);

        Ok(ProcessedPdf {
            full_text: text,
            chunks,
        })
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of PDF processing containing extracted text and chunks
#[derive(Debug, Clone)]
pub struct ProcessedPdf {
    pub full_text: String,
    pub chunks: Vec<String>,
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
pub fn generate_pdf_filename(game_id: crate::models::GameId, original_filename: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
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
        let service = Processor::new();
        let text = "This is a test text that should be chunked. ".repeat(50);
        let chunks = service.chunk_text(&text);

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
        let game_id: crate::models::GameId = 123;
        let original = "My Game Rules.pdf";
        let filename = generate_pdf_filename(game_id, original);

        assert!(filename.starts_with("game_123_"));
        assert!(filename.ends_with(".pdf"));
    }

    #[tokio::test]
    async fn test_process_pdf_with_nonexistent_file() {
        let service = Processor::new();

        // Test with nonexistent file should return error
        match service.process_pdf(Path::new("nonexistent.pdf")).await {
            Ok(_) => {
                panic!("Expected error for nonexistent file");
            }
            Err(_) => {
                // Expected - file doesn't exist
            }
        }
    }

    #[test]
    fn test_empty_text_chunking() {
        let service = Processor::new();
        let chunks = service.chunk_text("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_short_text_chunking() {
        let service = Processor::new();
        let text = "Short text that shouldn't be chunked.";
        let chunks = service.chunk_text(text);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_long_text_chunking() {
        let service = Processor::new();
        let text = "A ".repeat(600); // 1200 characters
        let chunks = service.chunk_text(&text);

        assert!(chunks.len() >= 2); // Should be split into multiple chunks
        assert!(chunks[0].len() <= CHUNK_SIZE);
    }

    #[test]
    fn test_chunk_overlap() {
        let service = Processor::new();
        let text = "word ".repeat(300); // 1500 characters
        let chunks = service.chunk_text(&text);

        if chunks.len() >= 2 {
            // Check that there's some overlap between chunks
            let chunk1_end = &chunks[0][(chunks[0].len() - 50)..];
            let chunk2_start = &chunks[1][..50];

            // There should be some common words due to overlap
            let chunk1_words: Vec<&str> = chunk1_end.split_whitespace().collect();
            let chunk2_words: Vec<&str> = chunk2_start.split_whitespace().collect();

            let has_overlap = chunk1_words.iter().any(|word| chunk2_words.contains(word));

            assert!(has_overlap, "Chunks should have overlapping content");
        }
    }

    #[test]
    fn test_whitespace_cleanup() {
        let service = Processor::new();
        let text_with_extra_whitespace = "Line 1\n\n  \n\nLine 2\n   \n\n\nLine 3";
        let chunks = service.chunk_text(text_with_extra_whitespace);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Line 1 Line 2 Line 3");
    }
}
