use dropshot::{HttpError, Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::AppState;
use crate::auth::require_auth;
use crate::db::challenges;
use crate::models::{
    AddParticipantRequest, AssignGameRequest, Challenge, ChallengeGame, ChallengeGridView,
    ChallengeParticipant, ChallengePlayWithParticipants, ChallengeStats, ChallengeSummary,
    CreateChallengeRequest, PaginatedResponse, PaginationParams, RecordPlayRequest,
    UpdateChallengeRequest, UpdatePlayRequest,
};

use super::{
    HttpCreated, HttpDeleted, HttpOk, bad_request_error, created_response, deleted_response,
    forbidden_error, internal_error, not_found_error, success_response,
};
use crate::db::Database;

/// Helper to verify user is a participant in a challenge
async fn require_participant(
    db: &Database,
    challenge_id: i64,
    user_id: i64,
) -> Result<(), HttpError> {
    let is_participant = challenges::is_participant(db, challenge_id, user_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !is_participant {
        return Err(forbidden_error(
            "You are not a participant in this challenge".to_string(),
        ));
    }
    Ok(())
}

/// Helper to verify user is the owner of a challenge
async fn require_owner(db: &Database, challenge_id: i64, user_id: i64) -> Result<(), HttpError> {
    let is_owner = challenges::is_owner(db, challenge_id, user_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !is_owner {
        return Err(forbidden_error(
            "Only the owner can perform this action".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChallengePath {
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChallengeGamePath {
    pub id: i64,
    pub game_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChallengeParticipantPath {
    pub id: i64,
    pub user_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChallengePlayPath {
    pub id: i64,
    pub play_id: i64,
}

/// List current user's challenges
#[endpoint {
    method = GET,
    path = "/api/challenges",
    tags = ["challenges"]
}]
pub async fn list_challenges(
    rqctx: RequestContext<AppState>,
    query: Query<PaginationParams>,
) -> Result<HttpOk<PaginatedResponse<ChallengeSummary>>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let query = query.into_inner();

    let (items, total) =
        challenges::list_user_challenges(&db, user.user_id, query.page, query.limit)
            .await
            .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    success_response(PaginatedResponse::new(
        items,
        total,
        query.page,
        query.limit,
    ))
}

/// Create a new challenge
#[endpoint {
    method = POST,
    path = "/api/challenges",
    tags = ["challenges"]
}]
pub async fn create_challenge(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateChallengeRequest>,
) -> Result<HttpCreated<Challenge>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let request = body.into_inner();

    // Validate grid dimensions
    if request.grid_rows < 1 || request.grid_rows > 10 {
        return Err(bad_request_error(
            "grid_rows must be between 1 and 10".to_string(),
        ));
    }
    if request.grid_cols < 1 || request.grid_cols > 10 {
        return Err(bad_request_error(
            "grid_cols must be between 1 and 10".to_string(),
        ));
    }

    let challenge = challenges::create_challenge(&db, user.user_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    created_response(challenge)
}

/// Get a challenge by ID
#[endpoint {
    method = GET,
    path = "/api/challenges/{id}",
    tags = ["challenges"]
}]
pub async fn get_challenge(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
) -> Result<HttpOk<Challenge>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;

    require_participant(&db, challenge_id, user.user_id).await?;

    let challenge = challenges::get_challenge(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Challenge not found".to_string()))?;

    success_response(challenge)
}

/// Update a challenge
#[endpoint {
    method = PATCH,
    path = "/api/challenges/{id}",
    tags = ["challenges"]
}]
pub async fn update_challenge(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
    body: TypedBody<UpdateChallengeRequest>,
) -> Result<HttpOk<Challenge>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;
    let request = body.into_inner();

    require_owner(&db, challenge_id, user.user_id).await?;

    let challenge = challenges::update_challenge(&db, challenge_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Challenge not found".to_string()))?;

    success_response(challenge)
}

/// Delete a challenge
#[endpoint {
    method = DELETE,
    path = "/api/challenges/{id}",
    tags = ["challenges"]
}]
pub async fn delete_challenge(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
) -> Result<HttpDeleted, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;

    require_owner(&db, challenge_id, user.user_id).await?;

    let deleted = challenges::delete_challenge(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !deleted {
        return Err(not_found_error("Challenge not found".to_string()));
    }

    deleted_response()
}

/// Get full grid view of a challenge
#[endpoint {
    method = GET,
    path = "/api/challenges/{id}/grid",
    tags = ["challenges"]
}]
pub async fn get_challenge_grid(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
) -> Result<HttpOk<ChallengeGridView>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;

    require_participant(&db, challenge_id, user.user_id).await?;

    let challenge = challenges::get_challenge(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Challenge not found".to_string()))?;

    let participants = challenges::get_participants(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    let games = challenges::get_games(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    let plays = challenges::get_plays(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    let stats = challenges::get_stats(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    success_response(ChallengeGridView {
        challenge,
        participants,
        games,
        plays,
        stats,
    })
}

/// Add a participant to a challenge
#[endpoint {
    method = POST,
    path = "/api/challenges/{id}/participants",
    tags = ["challenges"]
}]
pub async fn add_participant(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
    body: TypedBody<AddParticipantRequest>,
) -> Result<HttpCreated<ChallengeParticipant>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;
    let request = body.into_inner();

    require_owner(&db, challenge_id, user.user_id).await?;

    let participant = challenges::add_participant(&db, challenge_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    created_response(participant)
}

/// Remove a participant from a challenge
#[endpoint {
    method = DELETE,
    path = "/api/challenges/{id}/participants/{user_id}",
    tags = ["challenges"]
}]
pub async fn remove_participant(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengeParticipantPath>,
) -> Result<HttpDeleted, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let path = path.into_inner();

    require_owner(&db, path.id, user.user_id).await?;

    let removed = challenges::remove_participant(&db, path.id, path.user_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !removed {
        return Err(bad_request_error(
            "Cannot remove the owner or participant not found".to_string(),
        ));
    }

    deleted_response()
}

/// Assign a game to a challenge row
#[endpoint {
    method = POST,
    path = "/api/challenges/{id}/games",
    tags = ["challenges"]
}]
pub async fn assign_game(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
    body: TypedBody<AssignGameRequest>,
) -> Result<HttpCreated<ChallengeGame>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;
    let request = body.into_inner();

    require_participant(&db, challenge_id, user.user_id).await?;

    // Validate row_index is within grid bounds
    let challenge = challenges::get_challenge(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Challenge not found".to_string()))?;

    if request.row_index < 0 || request.row_index >= challenge.grid_rows {
        return Err(bad_request_error(format!(
            "row_index must be between 0 and {}",
            challenge.grid_rows - 1
        )));
    }

    let game = challenges::assign_game(&db, challenge_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    created_response(game)
}

/// Remove a game from a challenge
#[endpoint {
    method = DELETE,
    path = "/api/challenges/{id}/games/{game_id}",
    tags = ["challenges"]
}]
pub async fn remove_game(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengeGamePath>,
) -> Result<HttpDeleted, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let path = path.into_inner();

    require_participant(&db, path.id, user.user_id).await?;

    let removed = challenges::remove_game(&db, path.id, path.game_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !removed {
        return Err(not_found_error("Game not found in challenge".to_string()));
    }

    deleted_response()
}

/// Record a play in a challenge
#[endpoint {
    method = POST,
    path = "/api/challenges/{id}/plays",
    tags = ["challenges"]
}]
pub async fn record_play(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
    body: TypedBody<RecordPlayRequest>,
) -> Result<HttpCreated<ChallengePlayWithParticipants>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;
    let request = body.into_inner();

    // Check if user is a participant
    require_participant(&db, challenge_id, user.user_id).await?;

    // Validate col_index is within grid bounds
    let challenge = challenges::get_challenge(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Challenge not found".to_string()))?;

    if request.col_index < 0 || request.col_index >= challenge.grid_cols {
        return Err(bad_request_error(format!(
            "col_index must be between 0 and {}",
            challenge.grid_cols - 1
        )));
    }

    // Validate challenge_game_id belongs to this challenge
    let game_valid =
        challenges::game_belongs_to_challenge(&db, challenge_id, request.challenge_game_id)
            .await
            .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !game_valid {
        return Err(bad_request_error(
            "Invalid game for this challenge".to_string(),
        ));
    }

    // Validate all play participants are challenge participants
    let participant_user_ids: Vec<i64> = request.participants.iter().map(|p| p.user_id).collect();
    let participants_valid =
        challenges::validate_play_participants(&db, challenge_id, &participant_user_ids)
            .await
            .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !participants_valid {
        return Err(bad_request_error(
            "All play participants must be challenge participants".to_string(),
        ));
    }

    let play = challenges::record_play(&db, challenge_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    created_response(play)
}

/// Update a play
#[endpoint {
    method = PATCH,
    path = "/api/challenges/{id}/plays/{play_id}",
    tags = ["challenges"]
}]
pub async fn update_play(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePlayPath>,
    body: TypedBody<UpdatePlayRequest>,
) -> Result<HttpOk<ChallengePlayWithParticipants>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let path = path.into_inner();
    let request = body.into_inner();

    // Check if user is a participant
    require_participant(&db, path.id, user.user_id).await?;

    // Verify the play belongs to this challenge
    let play_valid = challenges::play_belongs_to_challenge(&db, path.id, path.play_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !play_valid {
        return Err(not_found_error(
            "Play not found in this challenge".to_string(),
        ));
    }

    // Validate all play participants are challenge participants if provided
    if let Some(ref participants) = request.participants {
        let participant_user_ids: Vec<i64> = participants.iter().map(|p| p.user_id).collect();
        let participants_valid =
            challenges::validate_play_participants(&db, path.id, &participant_user_ids)
                .await
                .map_err(|e| internal_error(format!("Database error: {}", e)))?;

        if !participants_valid {
            return Err(bad_request_error(
                "All play participants must be challenge participants".to_string(),
            ));
        }
    }

    let play = challenges::update_play(&db, path.play_id, request)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| not_found_error("Play not found".to_string()))?;

    success_response(play)
}

/// Delete a play
#[endpoint {
    method = DELETE,
    path = "/api/challenges/{id}/plays/{play_id}",
    tags = ["challenges"]
}]
pub async fn delete_play(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePlayPath>,
) -> Result<HttpDeleted, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let path = path.into_inner();

    // Check if user is a participant
    require_participant(&db, path.id, user.user_id).await?;

    // Verify the play belongs to this challenge
    let play_valid = challenges::play_belongs_to_challenge(&db, path.id, path.play_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !play_valid {
        return Err(not_found_error(
            "Play not found in this challenge".to_string(),
        ));
    }

    let deleted = challenges::delete_play(&db, path.play_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    if !deleted {
        return Err(not_found_error("Play not found".to_string()));
    }

    deleted_response()
}

/// Get challenge stats and leaderboard
#[endpoint {
    method = GET,
    path = "/api/challenges/{id}/stats",
    tags = ["challenges"]
}]
pub async fn get_challenge_stats(
    rqctx: RequestContext<AppState>,
    path: Path<ChallengePath>,
) -> Result<HttpOk<ChallengeStats>, HttpError> {
    let user = require_auth(&rqctx)?;
    let db = rqctx.context().db();
    let challenge_id = path.into_inner().id;

    require_participant(&db, challenge_id, user.user_id).await?;

    let stats = challenges::get_stats(&db, challenge_id)
        .await
        .map_err(|e| internal_error(format!("Database error: {}", e)))?;

    success_response(stats)
}
