use anyhow::{Result, anyhow};
use pdf_extract::extract_text;
use std::path::Path;

/// Configuration for text chunking
const CHUNK_SIZE: usize = 1000; // characters per chunk
const CHUNK_OVERLAP: usize = 300; // overlap between chunks
const MIN_CHUNK_SIZE: usize = 100; // minimum characters for a valid chunk
const MAX_CHUNK_SIZE: usize = 1500; // maximum characters before forced split

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

    /// Split text into chunks for embedding with intelligent sentence boundary detection
    pub fn chunk_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();

        if text.trim().is_empty() {
            return chunks;
        }

        // Clean and normalize the text first
        let cleaned_text = self.clean_text(text);

        // Split into sentences first for better boundary detection
        let sentences = self.split_into_sentences(&cleaned_text);

        if sentences.is_empty() {
            return chunks;
        }

        let mut current_chunk = String::new();
        let mut sentence_buffer = Vec::new();

        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            // Check if adding this sentence would exceed max size
            let would_exceed = !current_chunk.is_empty()
                && (current_chunk.len() + sentence.len() + 1) > MAX_CHUNK_SIZE;

            if would_exceed && current_chunk.len() >= MIN_CHUNK_SIZE {
                // Finalize current chunk
                chunks.push(current_chunk.trim().to_string());

                // Start new chunk with sentence overlap for context
                current_chunk = self.create_sentence_overlap(&sentence_buffer);
                sentence_buffer.clear();
            }

            // Add sentence to current chunk
            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(sentence);
            sentence_buffer.push(sentence.to_string());

            // If we've reached a good chunk size and have complete sentences, consider chunking
            if current_chunk.len() >= CHUNK_SIZE && self.is_good_chunk_boundary(&sentence) {
                chunks.push(current_chunk.trim().to_string());

                // Start new chunk with overlap
                current_chunk = self.create_sentence_overlap(&sentence_buffer);
                sentence_buffer.clear();
            }
        }

        // Add the final chunk if it has content
        if current_chunk.trim().len() >= MIN_CHUNK_SIZE {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    /// Clean and normalize text for better processing
    fn clean_text(&self, text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .collect::<String>()
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    /// Split text into sentences with proper boundary detection
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            current_sentence.push(ch);

            // Check for sentence endings
            if ".!?".contains(ch) {
                // Look ahead to see if this is really a sentence end
                if self.is_sentence_end(&chars, i) {
                    // Include any trailing punctuation/quotes
                    i += 1;
                    while i < chars.len() {
                        let next_ch = chars[i];
                        if "\"')]} \t".contains(next_ch) {
                            if !" \t".contains(next_ch) {
                                current_sentence.push(next_ch);
                            }
                            i += 1;
                        } else {
                            break;
                        }
                    }

                    let sentence = current_sentence.trim().to_string();
                    if !sentence.is_empty() && sentence.len() > 10 {
                        sentences.push(sentence);
                    }
                    current_sentence.clear();
                    continue;
                }
            }

            i += 1;
        }

        // Add any remaining content as the last sentence
        let final_sentence = current_sentence.trim().to_string();
        if !final_sentence.is_empty() && final_sentence.len() > 10 {
            sentences.push(final_sentence);
        }

        sentences
    }

    /// Check if a position represents the end of a sentence
    fn is_sentence_end(&self, chars: &[char], pos: usize) -> bool {
        let ch = chars[pos];

        // Must be a sentence ending punctuation
        if !".!?".contains(ch) {
            return false;
        }

        // Check for common abbreviations that aren't sentence ends
        if ch == '.' {
            // Look backward for common abbreviations
            let start = pos.saturating_sub(10);
            let before: String = chars[start..pos].iter().collect();
            let before_lower = before.to_lowercase();

            // Common abbreviations in game rules
            let abbreviations = [
                "vs", "etc", "e.g", "i.e", "mr", "mrs", "dr", "no", "inc", "ltd",
            ];
            for abbrev in &abbreviations {
                if before_lower.ends_with(abbrev) {
                    return false;
                }
            }

            // Check for numbered items like "1." "2." etc.
            if pos > 0 && chars[pos - 1].is_numeric() {
                // Look back to see if it's just a number
                let mut j = pos - 1;
                while j > 0 && (chars[j].is_numeric() || chars[j] == ' ') {
                    j -= 1;
                }
                // If we find a newline or start of text, this might be a list item
                if j == 0 || chars[j] == '\n' {
                    return false;
                }
            }
        }

        // Look ahead to see what follows
        let mut next_pos = pos + 1;

        // Skip whitespace and quotes
        while next_pos < chars.len() && " \t\n\"')]}".contains(chars[next_pos]) {
            next_pos += 1;
        }

        // If we're at the end, it's a sentence end
        if next_pos >= chars.len() {
            return true;
        }

        // If next character is uppercase or start of new paragraph, likely sentence end
        let next_char = chars[next_pos];
        next_char.is_uppercase() || next_char.is_numeric()
    }

    /// Check if this is a good place to end a chunk (complete thought)
    fn is_good_chunk_boundary(&self, sentence: &str) -> bool {
        let sentence_lower = sentence.to_lowercase();

        // Good boundaries are sentences that complete a thought
        // Prefer ending after sentences that contain conclusions, rules, or complete instructions
        sentence_lower.contains("therefore") ||
        sentence_lower.contains("thus") ||
        sentence_lower.contains("finally") ||
        sentence_lower.contains("in conclusion") ||
        sentence_lower.ends_with("wins.") ||
        sentence_lower.ends_with("wins!") ||
        sentence_lower.ends_with("complete.") ||
        sentence_lower.ends_with("finished.") ||

        // Or avoid breaking in the middle of procedures
        !(sentence_lower.contains("first") ||
          sentence_lower.contains("then") ||
          sentence_lower.contains("next") ||
          sentence_lower.contains("after") ||
          sentence_lower.contains("before") ||
          sentence_lower.ends_with(":"))
    }

    /// Create overlap text from previous sentences for context continuity
    fn create_sentence_overlap(&self, sentences: &[String]) -> String {
        if sentences.is_empty() {
            return String::new();
        }

        let mut overlap = String::new();
        let mut current_length = 0;
        let target_overlap = CHUNK_OVERLAP;

        // Take the last few sentences to create meaningful overlap
        for sentence in sentences.iter().rev() {
            if current_length + sentence.len() <= target_overlap {
                if overlap.is_empty() {
                    overlap = sentence.clone();
                } else {
                    overlap = format!("{} {}", sentence, overlap);
                }
                current_length += sentence.len() + 1;
            } else {
                break;
            }
        }

        overlap
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
        assert!(chunks[0].len() <= MAX_CHUNK_SIZE);
        assert!(chunks[0].len() >= MIN_CHUNK_SIZE);
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
        let text = "This is a short text that should create one chunk since it's long enough to meet the minimum chunk size requirement for our sentence-aware chunking system.";
        let chunks = service.chunk_text(text);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_long_text_chunking() {
        let service = Processor::new();
        let text = "A ".repeat(600); // 1200 characters
        let chunks = service.chunk_text(&text);

        assert!(chunks.len() >= 1); // Should be split appropriately
        for chunk in &chunks {
            assert!(chunk.len() <= MAX_CHUNK_SIZE);
            assert!(chunk.len() >= MIN_CHUNK_SIZE);
        }
    }

    #[test]
    fn test_sentence_aware_chunking() {
        let service = Processor::new();
        let text = "This is the first sentence. This is the second sentence. ".repeat(20)
            + "This is a final sentence that should be preserved.";
        let chunks = service.chunk_text(&text);

        // Chunks should not end mid-sentence when possible
        for chunk in &chunks {
            let trimmed = chunk.trim();
            if trimmed.len() > 100 {
                // Only check substantial chunks
                assert!(
                    trimmed.ends_with('.')
                        || trimmed.ends_with('!')
                        || trimmed.ends_with('?')
                        || chunks.len() == 1, // Single chunk case
                    "Chunk should end at sentence boundary: '{}'",
                    &trimmed[trimmed.len().saturating_sub(50)..]
                );
            }
        }
    }

    #[test]
    fn test_sentence_splitting() {
        let service = Processor::new();

        // Test basic sentence splitting
        let text = "First sentence. Second sentence! Third sentence?";
        let sentences = service.split_into_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "First sentence.");
        assert_eq!(sentences[1], "Second sentence!");
        assert_eq!(sentences[2], "Third sentence?");

        // Test abbreviation handling
        let text_with_abbrev = "Use vs. mode for competition. This is separate.";
        let sentences_abbrev = service.split_into_sentences(text_with_abbrev);
        assert_eq!(sentences_abbrev.len(), 2);

        // Test numbered lists (should not split)
        let numbered_text = "Follow these steps: 1. Setup the board. 2. Deal cards.";
        let numbered_sentences = service.split_into_sentences(numbered_text);
        // The numbers shouldn't create false sentence breaks
        assert!(numbered_sentences.len() <= 3);
    }

    #[test]
    fn test_sentence_boundary_detection() {
        let service = Processor::new();

        // Test normal sentence end
        let chars: Vec<char> = "Hello world. Next sentence".chars().collect();
        assert!(service.is_sentence_end(&chars, 11)); // Position of '.'

        // Test abbreviation (should not be sentence end)
        let abbrev_chars: Vec<char> = "Use vs. other players".chars().collect();
        assert!(!service.is_sentence_end(&abbrev_chars, 6)); // Position of '.'

        // Test numbered item (should not be sentence end)
        let numbered_chars: Vec<char> = "1. First item".chars().collect();
        assert!(!service.is_sentence_end(&numbered_chars, 1)); // Position of '.'
    }

    #[test]
    fn test_chunk_boundary_detection() {
        let service = Processor::new();

        // Good boundaries
        assert!(service.is_good_chunk_boundary("The player wins the game."));
        assert!(service.is_good_chunk_boundary("This completes the setup."));

        // Poor boundaries (in middle of procedure)
        assert!(!service.is_good_chunk_boundary("First, place the board:"));
        assert!(!service.is_good_chunk_boundary("Then move your piece."));
    }

    #[test]
    fn test_whitespace_cleanup() {
        let service = Processor::new();
        let text_with_extra_whitespace = "Line 1 contains enough text to meet minimum requirements.\n\n  \n\nLine 2 also has sufficient content for processing.\n   \n\n\nLine 3 completes our test with adequate length.";
        let chunks = service.chunk_text(text_with_extra_whitespace);

        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0],
            "Line 1 contains enough text to meet minimum requirements. Line 2 also has sufficient content for processing. Line 3 completes our test with adequate length."
        );
    }
}
