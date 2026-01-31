use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub num_players: usize,
    pub deck_seed: String,
    pub player_order: Vec<Uuid>,
    pub configuration: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_game_metadata_creation() {
        let player_order = vec![Uuid::new_v4(), Uuid::new_v4()];
        let metadata = GameMetadata {
            started_at: Utc::now(),
            num_players: 2,
            deck_seed: "test_seed".to_string(),
            player_order: player_order.clone(),
            configuration: Some(serde_json::json!({"key": "value"})),
        };

        assert_eq!(metadata.num_players, 2);
        assert_eq!(metadata.deck_seed, "test_seed");
        assert_eq!(metadata.player_order.len(), 2);
        assert!(metadata.configuration.is_some());
    }
}
