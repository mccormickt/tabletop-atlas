use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{GameId, UserId};

pub type CollectionEntryId = i64;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CollectionEntry {
    pub id: CollectionEntryId,
    pub user_id: UserId,
    pub master_game_id: GameId,
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub play_count: i32,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CollectionEntryWithGame {
    pub id: CollectionEntryId,
    pub master_game_id: GameId,
    pub game_name: String,
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub play_count: i32,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddToCollectionRequest {
    pub master_game_id: GameId,
    pub notes: Option<String>,
    pub rating: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCollectionRequest {
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub play_count: Option<i32>,
}
