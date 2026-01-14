use chrono::{DateTime, NaiveDate, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::UserId;

pub type ChallengeId = i64;
pub type ChallengeGameId = i64;
pub type ChallengePlayId = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeStatus {
    Draft,
    Active,
    Completed,
    Archived,
}

impl std::fmt::Display for ChallengeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChallengeStatus::Draft => write!(f, "draft"),
            ChallengeStatus::Active => write!(f, "active"),
            ChallengeStatus::Completed => write!(f, "completed"),
            ChallengeStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for ChallengeStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(ChallengeStatus::Draft),
            "active" => Ok(ChallengeStatus::Active),
            "completed" => Ok(ChallengeStatus::Completed),
            "archived" => Ok(ChallengeStatus::Archived),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum GameType {
    Master,
    Custom,
    Collection,
}

impl std::fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::Master => write!(f, "master"),
            GameType::Custom => write!(f, "custom"),
            GameType::Collection => write!(f, "collection"),
        }
    }
}

impl std::str::FromStr for GameType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "master" => Ok(GameType::Master),
            "custom" => Ok(GameType::Custom),
            "collection" => Ok(GameType::Collection),
            _ => Err(format!("Invalid game type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantRole {
    Owner,
    Participant,
}

impl std::fmt::Display for ParticipantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticipantRole::Owner => write!(f, "owner"),
            ParticipantRole::Participant => write!(f, "participant"),
        }
    }
}

impl std::str::FromStr for ParticipantRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(ParticipantRole::Owner),
            "participant" => Ok(ParticipantRole::Participant),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

// Main challenge entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: ChallengeId,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: UserId,
    pub grid_rows: i32,
    pub grid_cols: i32,
    pub status: ChallengeStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Challenge participant
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeParticipant {
    pub id: i64,
    pub challenge_id: ChallengeId,
    pub user_id: UserId,
    pub role: ParticipantRole,
    pub joined_at: DateTime<Utc>,
    pub display_name: Option<String>,
    pub picture_url: Option<String>,
}

// Challenge game (row assignment)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeGame {
    pub id: ChallengeGameId,
    pub challenge_id: ChallengeId,
    pub row_index: i32,
    pub game_type: GameType,
    pub game_id: i64,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Challenge play (cell)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChallengePlay {
    pub id: ChallengePlayId,
    pub challenge_id: ChallengeId,
    pub challenge_game_id: ChallengeGameId,
    pub col_index: i32,
    pub played_at: NaiveDate,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Play participant
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayParticipant {
    pub id: i64,
    pub challenge_play_id: ChallengePlayId,
    pub user_id: UserId,
    pub is_winner: bool,
    pub score: Option<i32>,
    pub display_name: Option<String>,
}

// Request/Response types

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChallengeRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_grid_size")]
    pub grid_rows: i32,
    #[serde(default = "default_grid_size")]
    pub grid_cols: i32,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

fn default_grid_size() -> i32 {
    8
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChallengeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ChallengeStatus>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddParticipantRequest {
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssignGameRequest {
    pub row_index: i32,
    pub game_type: GameType,
    pub game_id: i64,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayParticipantInput {
    pub user_id: UserId,
    pub is_winner: bool,
    pub score: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordPlayRequest {
    pub challenge_game_id: ChallengeGameId,
    pub col_index: i32,
    pub played_at: NaiveDate,
    pub notes: Option<String>,
    pub participants: Vec<PlayParticipantInput>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayRequest {
    pub played_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub participants: Option<Vec<PlayParticipantInput>>,
}

// Response types with joined data

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChallengePlayWithParticipants {
    pub id: ChallengePlayId,
    pub challenge_id: ChallengeId,
    pub challenge_game_id: ChallengeGameId,
    pub col_index: i32,
    pub played_at: NaiveDate,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub participants: Vec<PlayParticipant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeGridView {
    pub challenge: Challenge,
    pub participants: Vec<ChallengeParticipant>,
    pub games: Vec<ChallengeGame>,
    pub plays: Vec<ChallengePlayWithParticipants>,
    pub stats: ChallengeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeStats {
    pub total_cells: i32,
    pub completed_cells: i32,
    pub completion_percentage: f64,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub picture_url: Option<String>,
    pub wins: i32,
    pub total_plays: i32,
    pub win_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeSummary {
    pub id: ChallengeId,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: UserId,
    pub grid_rows: i32,
    pub grid_cols: i32,
    pub status: ChallengeStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub participant_count: i32,
    pub completion_percentage: f64,
    pub created_at: DateTime<Utc>,
}
