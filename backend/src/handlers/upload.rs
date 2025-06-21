use std::fs;
use std::path::PathBuf;

use dropshot::{Path, RequestContext, UntypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{bad_request_error, internal_error, not_found_error, success_response};
use crate::{
    AppState, db,
    handlers::{HttpError, HttpOk},
    models::{CreateEmbeddingRequest, EmbeddingSourceType, GameId, RulesInfoResponse},
    pdf::{Processor, generate_pdf_filename, validate_pdf_file},
};

#[derive(Deserialize, JsonSchema)]
pub struct UploadPathParam {
    pub id: GameId,
}

#[derive(Serialize, JsonSchema)]
pub struct UploadResponse {
    pub message: String,
    pub file_path: Option<String>,
    pub chunks_processed: Option<u32>,
    pub text_length: Option<usize>,
}

/// Upload a PDF rules document for a game
#[endpoint {
    method = POST,
    path = "/api/games/{id}/rules-upload"
}]
pub async fn upload_rules_pdf(
    rqctx: RequestContext<AppState>,
    path: Path<UploadPathParam>,
    body: UntypedBody,
) -> Result<HttpOk<UploadResponse>, HttpError> {
    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let body_bytes = body.as_bytes();

    // Validate that we have data
    if body_bytes.is_empty() {
        return Err(bad_request_error("No file data provided".to_string()));
    }

    // Validate that the file is a PDF
    if let Err(e) = validate_pdf_file(body_bytes) {
        return Err(bad_request_error(format!("Invalid PDF file: {}", e)));
    }

    // Check if the game exists
    let db = app_state.db();
    let game = db::games::get_game(&db, game_id)
        .await
        .map_err(|e| internal_error(format!("Failed to get game: {}", e)))?
        .ok_or(not_found_error(format!(
            "Game with id {} not found",
            game_id as i64
        )))?;

    // Create uploads directory if it doesn't exist
    let uploads_dir = PathBuf::from("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(&uploads_dir)
            .map_err(|e| internal_error(format!("Failed to create uploads directory: {}", e)))?;
    }

    // Generate a unique filename
    let filename = generate_pdf_filename(game.id, "rules.pdf");
    let file_path = uploads_dir.join(&filename);

    // Save the file
    fs::write(&file_path, body_bytes)
        .map_err(|e| internal_error(format!("Failed to save file: {}", e)))?;

    // Process PDF: extract text and create chunks
    let pdf_service = Processor::new();
    let processed_pdf = pdf_service.process_pdf(&file_path).await.map_err(|e| {
        let _ = fs::remove_file(&file_path);
        internal_error(format!("Failed to extract PDF text: {}", e))
    })?;

    // Generate embeddings for all chunks
    let embeddings = app_state
        .embedder()
        .generate_embeddings(&processed_pdf.chunks)
        .await
        .map_err(|e| {
            let _ = fs::remove_file(&file_path);
            internal_error(format!("Failed to generate embeddings: {}", e))
        })?;

    // Create embedding requests for database storage
    let embedding_requests: Vec<CreateEmbeddingRequest> = processed_pdf
        .chunks
        .iter()
        .zip(embeddings.iter())
        .enumerate()
        .map(|(chunk_index, (chunk, embedding))| {
            let metadata = serde_json::json!({
                "file_name": &filename,
                "chunk_size": chunk.len(),
                "total_chunks": processed_pdf.chunks.len(),
                "processing_timestamp": chrono::Utc::now().to_rfc3339(),
                "embedding_model": app_state.embedder().get_model()
            });

            CreateEmbeddingRequest {
                game_id: game.id,
                chunk_text: chunk.clone(),
                embedding: embedding.clone(),
                chunk_index: chunk_index as i32,
                source_type: EmbeddingSourceType::RulesPdf,
                source_id: None,
                metadata: Some(metadata.to_string()),
            }
        })
        .collect();

    // Update game with rules text
    db::games::update_game_rules_text(
        &db,
        game.id,
        processed_pdf.full_text.clone(),
        Some(file_path.to_string_lossy().to_string()),
    )
    .await
    .map_err(|e| {
        let _ = fs::remove_file(&file_path);
        internal_error(format!("Failed to update game rules text: {}", e))
    })?;

    // Store embeddings in batch
    crate::db::embeddings::create_embeddings_batch(&db, embedding_requests.clone())
        .await
        .map_err(|e| {
            let _ = fs::remove_file(&file_path);
            internal_error(format!("Failed to store embeddings: {}", e))
        })?;

    let response = UploadResponse {
        message: format!(
            "Successfully uploaded and processed PDF for game {}. Extracted {} characters and created {} text chunks.",
            game_id as i64,
            processed_pdf.full_text.len(),
            processed_pdf.chunks.len()
        ),
        file_path: Some(file_path.to_string_lossy().to_string()),
        chunks_processed: Some(processed_pdf.chunks.len() as u32),
        text_length: Some(processed_pdf.full_text.len()),
    };

    success_response(response)
}

/// Get information about uploaded rules for a game
#[endpoint {
    method = GET,
    path = "/api/games/{id}/rules-info"
}]
pub async fn get_rules_info(
    rqctx: RequestContext<AppState>,
    path: Path<UploadPathParam>,
) -> Result<HttpOk<RulesInfoResponse>, HttpError> {
    let app_state = rqctx.context();
    let game_id = path.into_inner().id;
    let db = app_state.db();

    // Get game rules info using consolidated database function
    let result = db::games::get_game_rules_info(&db, game_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or(not_found_error(format!(
            "Game with id {} not found",
            game_id as i64
        )))?;

    success_response(result)
}

/// Delete uploaded rules for a game
#[endpoint {
    method = DELETE,
    path = "/api/games/{id}/rules"
}]
pub async fn delete_rules(
    rqctx: RequestContext<AppState>,
    path: Path<UploadPathParam>,
) -> Result<HttpOk<DeleteRulesResponse>, HttpError> {
    let app_state = rqctx.context();
    let game_id = path.into_inner().id;

    let db = app_state.db();

    // Get the current PDF path before deletion
    let pdf_path: Option<String> = db
        .with_connection(|conn| {
            conn.query_row(
                "SELECT rules_pdf_path FROM games WHERE id = ?",
                [game_id as i64],
                |row| row.get(0),
            )
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                not_found_error(format!("Game with id {} not found", game_id as i64))
            }
            _ => internal_error(format!("Database error: {}", e)),
        })?;

    // Delete embeddings associated with this game's PDF using consolidated function
    let embeddings_deleted = crate::db::embeddings::delete_embeddings_for_game(
        &db,
        game_id,
        Some(EmbeddingSourceType::RulesPdf),
    )
    .await
    .map_err(|e| internal_error(format!("Failed to delete embeddings: {}", e)))?;

    // Clear the PDF path and rules text from the game record
    db.with_connection(|conn| {
        conn.execute(
            "UPDATE games SET rules_pdf_path = NULL, rules_text = NULL WHERE id = ?",
            [game_id as i64],
        )
    })
    .map_err(|e| internal_error(format!("Failed to update game record: {}", e)))?;

    // Try to delete the physical file if it exists
    let file_deleted = if let Some(path) = pdf_path {
        let file_path = PathBuf::from(&path);
        if file_path.exists() {
            fs::remove_file(&file_path).is_ok()
        } else {
            false
        }
    } else {
        false
    };

    let response = DeleteRulesResponse {
        message: format!(
            "Successfully deleted rules for game {}. Removed {} embedding chunks.",
            game_id as i64, embeddings_deleted
        ),
        embeddings_deleted: embeddings_deleted as u32,
        file_deleted,
    };

    success_response(response)
}

#[derive(Serialize, JsonSchema)]
pub struct DeleteRulesResponse {
    pub message: String,
    pub embeddings_deleted: u32,
    pub file_deleted: bool,
}
