use dropshot::{HttpError, Path, RequestContext, UntypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::success_response;
use crate::{AppState, handlers::HttpOk, models::GameId};

#[derive(Deserialize, JsonSchema)]
pub struct UploadPathParam {
    pub id: GameId,
}

#[derive(Serialize, JsonSchema)]
pub struct UploadResponse {
    pub message: String,
    pub file_path: Option<String>,
    pub chunks_processed: Option<u32>,
}

/// Upload a PDF rules document for a game
#[endpoint {
    method = POST,
    path = "/api/games/{id}/upload-rules"
}]
pub async fn upload_rules_pdf(
    rqctx: RequestContext<AppState>,
    path: Path<UploadPathParam>,
    body: UntypedBody,
) -> Result<HttpOk<UploadResponse>, HttpError> {
    let _app_state = rqctx.context();
    let _game_id = path.into_inner().id;
    let _body_bytes = body.as_bytes();

    // TODO: Implement PDF upload and processing
    // 1. Validate the uploaded file is a PDF
    // 2. Save the PDF to disk
    // 3. Extract text from PDF
    // 4. Chunk the text for embeddings
    // 5. Generate embeddings for each chunk
    // 6. Store embeddings in database
    // 7. Update game record with PDF path and extracted text

    let response = UploadResponse {
        message: "PDF upload endpoint - not yet implemented".to_string(),
        file_path: None,
        chunks_processed: None,
    };

    success_response(response)
}
