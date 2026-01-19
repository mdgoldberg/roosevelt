use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRecord {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub id: Option<i64>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub num_players: usize,
    pub deck_seed: String,
    pub player_order: Vec<Uuid>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResultRecord {
    pub id: Option<i64>,
    pub game_id: i64,
    pub player_id: Uuid,
    pub finishing_place: usize,
    pub finishing_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: Option<i64>,
    pub game_id: i64,
    pub player_id: Uuid,
    pub action_type: String,
    pub card_play: Option<serde_json::Value>,
    pub target_player_id: Option<Uuid>,
    pub turn_order: usize,
    pub phase: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedWrite {
    pub id: Option<i64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error_type: String,
    pub error_message: String,
    pub data: Option<serde_json::Value>,
}
