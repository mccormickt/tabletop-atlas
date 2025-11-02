# PDF Upload and Embedding Implementation

This document describes the implementation of PDF upload functionality for game rules with text extraction and vector embeddings using SQLite.

## Architecture Overview

The PDF upload feature consists of several components:

- **PDF Processing Module** (`backend/src/pdf_processor.rs`): Handles PDF text extraction, chunking, and embedding generation
- **Upload Handlers** (`backend/src/handlers/upload.rs`): REST API endpoints for uploading, managing, and searching PDFs
- **SQLite with sqlite-vec**: Vector database for storing and searching embeddings
- **Database Schema**: Tables for storing embeddings, metadata, and relationships

## Key Features

### 1. PDF Upload and Processing
- Upload PDF files for specific games via REST API
- Extract text content from PDFs using `pdf-extract` crate
- Chunk text into manageable segments (1000 characters with 200-character overlap)
- Generate vector embeddings for each text chunk
- Store embeddings in SQLite with metadata

### 2. Vector Search
- Search through PDF content using text similarity
- Retrieve relevant chunks based on user queries
- Mock similarity scoring (ready for real embedding models)

### 3. Database Integration
- Store embeddings as JSON-serialized vectors in SQLite
- Link embeddings to specific games
- Track processing metadata and timestamps

## API Endpoints

### Upload PDF Rules
```http
POST /api/games/{id}/rules-upload
Content-Type: application/octet-stream

<PDF binary data>
```

**Response:**
```json
{
  "message": "Successfully uploaded and processed PDF...",
  "file_path": "/path/to/uploaded/file.pdf",
  "chunks_processed": 15,
  "text_length": 12500
}
```

### Get Rules Information
```http
GET /api/games/{id}/rules-info
```

**Response:**
```json
{
  "game_id": 123,
  "game_name": "Gloomhaven",
  "has_rules_pdf": true,
  "rules_pdf_path": "/uploads/game_123_20240101_120000.pdf",
  "text_length": 12500,
  "chunk_count": 15,
  "last_processed": "2024-01-01T12:00:00Z"
}
```

### Search Rules
```http
GET /api/chat/search-rules?game_id=123&query=combat&limit=5
```

**Response:**
```json
{
  "game_id": 123,
  "query": "combat",
  "total_results": 3,
  "results": [
    {
      "chunk_id": 1,
      "chunk_text": "Combat in Gloomhaven follows a specific sequence...",
      "chunk_index": 0,
      "similarity_score": 0.8,
      "metadata": "{\"file_name\":\"rules.pdf\",\"chunk_size\":1000}"
    }
  ]
}
```

### Delete Rules
```http
DELETE /api/games/{id}/rules
```

**Response:**
```json
{
  "message": "Successfully deleted rules for game 123...",
  "embeddings_deleted": 15,
  "file_deleted": true
}
```

## Implementation Details

### PDF Processing Pipeline

1. **Validation**: Verify uploaded file is a valid PDF
2. **Text Extraction**: Use `pdf-extract` to extract plain text
3. **Chunking**: Split text into overlapping segments
4. **Embedding Generation**: Create vector representations (currently mock)
5. **Storage**: Save embeddings and metadata to SQLite

### Database Schema

The implementation uses several tables:

**embeddings table:**
- `id`: Primary key
- `game_id`: Foreign key to games table
- `chunk_text`: Original text chunk
- `embedding`: JSON-serialized vector (BLOB)
- `chunk_index`: Order within document
- `source_type`: 'rules_pdf' or 'house_rule'
- `metadata`: JSON metadata about the chunk

**games table extensions:**
- `rules_pdf_path`: Path to uploaded PDF file
- `rules_text`: Extracted text content

### Vector Storage

Currently, embeddings are stored as JSON-serialized vectors in SQLite BLOB fields. The implementation includes:

- 384-dimensional mock embeddings
- Normalized vectors for similarity calculations
- Metadata tracking for each chunk

### File Management

- PDFs are stored in `/uploads` directory
- Filenames include game ID and timestamp for uniqueness
- Automatic cleanup on processing failures
- File deletion when rules are removed

## Configuration

### Dependencies Added

```toml
# PDF text extraction
pdf-extract = "0.7"
# Vector embeddings for SQLite
sqlite-vec = "0.1"
# Zero-copy byte operations for vectors
zerocopy = "0.8"
```

### SQLite-vec Integration

The implementation initializes sqlite-vec in `main.rs`:

```rust
use sqlite_vec::sqlite3_vec_init;

unsafe {
    sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
}
```

## Usage Examples

### Upload a PDF
```bash
curl -X POST \
  http://localhost:8080/api/games/1/rules-upload \
  -H "Content-Type: application/octet-stream" \
  --data-binary @gloomhaven_rules.pdf
```

### Search Rules
```bash
curl "http://localhost:8080/api/chat/search-rules?game_id=1&query=combat&limit=5"
```

### Get Rules Info
```bash
curl http://localhost:8080/api/games/1/rules-info
```

## Future Enhancements

### Real Embedding Models
Replace mock embeddings with actual models:
- OpenAI text-embedding-ada-002
- Sentence Transformers
- Local embedding models

### Advanced Vector Search
- Implement proper vector similarity using sqlite-vec
- Add semantic search capabilities
- Support for multiple embedding models

### Enhanced Text Processing
- Better PDF parsing (handle tables, images)
- OCR support for scanned documents
- Improved text chunking strategies

### Metadata Enrichment
- Extract page numbers, sections, headings
- Support for structured documents
- Category tagging and classification

## Testing

The implementation includes basic tests for:
- PDF validation
- Text chunking
- Filename generation
- Embedding normalization

### Manual Testing
1. Start the server: `cargo run`
2. Upload a PDF using curl or API client
3. Verify embeddings are created in database
4. Test search functionality

## Error Handling

The implementation includes comprehensive error handling:
- Invalid PDF file validation
- File system operation errors
- Database transaction rollbacks
- Automatic cleanup on failures

## Performance Considerations

- Chunking is done in memory (suitable for typical rule books)
- Database operations use transactions
- File uploads are limited to 10MB by default
- Embeddings are generated sequentially (can be parallelized)

## Security Notes

- File uploads are validated as PDFs
- Uploaded files are stored outside web-accessible directories
- Database queries use parameterized statements
- File paths are sanitized

## Migration and Seeding

The implementation includes:
- V004 migration for seeding popular games from BoardGameGeek
- Automatic schema updates on server start
- Support for bulk game imports from CSV data

This implementation provides a solid foundation for PDF-based rule management and search functionality, ready for integration with LLM-powered chat features.