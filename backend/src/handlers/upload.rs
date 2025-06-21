use std::fs;
use std::path::PathBuf;

use dropshot::{Path, RequestContext, UntypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{bad_request_error, internal_error, not_found_error, success_response};
use crate::{
    AppState,
    db::Database,
    handlers::{HttpError, HttpOk},
    models::{EmbeddingSourceType, GameId},
    pdf_processor::{PdfProcessor, generate_pdf_filename, validate_pdf_file},
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
    {
        let db = app_state.db();
        let conn = db
            .lock()
            .map_err(|e| internal_error(format!("Database lock error: {}", e)))?;

        let mut stmt = conn
            .prepare("SELECT id FROM games WHERE id = ?")
            .map_err(|e| internal_error(format!("Database error: {}", e)))?;

        let game_exists = stmt
            .exists([game_id as i64])
            .map_err(|e| internal_error(format!("Database error: {}", e)))?;

        if !game_exists {
            return Err(not_found_error(format!(
                "Game with id {} not found",
                game_id as i64
            )));
        }
    }

    // Create uploads directory if it doesn't exist
    let uploads_dir = PathBuf::from("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(&uploads_dir)
            .map_err(|e| internal_error(format!("Failed to create uploads directory: {}", e)))?;
    }

    // Generate a unique filename
    let filename = generate_pdf_filename(game_id, "rules.pdf");
    let file_path = uploads_dir.join(&filename);

    // Save the file
    fs::write(&file_path, body_bytes)
        .map_err(|e| internal_error(format!("Failed to save file: {}", e)))?;

    // Process the PDF and store in database using consolidated functions
    let pdf_processor = PdfProcessor::new();
    let db = Database::new(app_state.db());

    let processing_result = pdf_processor
        .process_and_store_pdf(&db, game_id, &file_path, &filename)
        .await
        .map_err(|e| {
            // Clean up the file if processing fails
            let _ = fs::remove_file(&file_path);
            internal_error(format!("Failed to process PDF: {}", e))
        })?;

    let response = UploadResponse {
        message: format!(
            "Successfully uploaded and processed PDF for game {}. Extracted {} characters and created {} text chunks.",
            game_id as i64, processing_result.total_text_length, processing_result.chunks_processed
        ),
        file_path: Some(processing_result.file_path),
        chunks_processed: Some(processing_result.chunks_processed),
        text_length: Some(processing_result.total_text_length),
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
    let conn = db
        .lock()
        .map_err(|e| internal_error(format!("Database lock error: {}", e)))?;

    // Get game info and rules data
    let mut stmt = conn
        .prepare(
            r#"
        SELECT
            g.name,
            g.rules_pdf_path,
            g.rules_text,
            COUNT(e.id) as chunk_count,
            MAX(e.created_at) as last_processed
        FROM games g
        LEFT JOIN embeddings e ON g.id = e.game_id AND e.source_type = 'rules_pdf'
        WHERE g.id = ?
        GROUP BY g.id
        "#,
        )
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    let result = stmt
        .query_row([game_id as i64], |row| {
            Ok(RulesInfoResponse {
                game_id: game_id as i64,
                game_name: row.get(0)?,
                has_rules_pdf: row.get::<_, Option<String>>(1)?.is_some(),
                rules_pdf_path: row.get(1)?,
                text_length: row.get::<_, Option<String>>(2)?.map(|s| s.len()),
                chunk_count: row.get(3)?,
                last_processed: row.get(4)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                not_found_error(format!("Game with id {} not found", game_id as i64))
            }
            _ => internal_error(format!("Database error: {}", e)),
        })?;

    success_response(result)
}

#[derive(Serialize, JsonSchema)]
pub struct RulesInfoResponse {
    pub game_id: i64,
    pub game_name: String,
    pub has_rules_pdf: bool,
    pub rules_pdf_path: Option<String>,
    pub text_length: Option<usize>,
    pub chunk_count: i64,
    pub last_processed: Option<String>,
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

    let db = Database::new(app_state.db());

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
