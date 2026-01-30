use super::GameMetadata;
use crate::{ActionRecord, GameResultRecord};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GameEventCollector {
    pub metadata: GameMetadata,
    pub actions: Vec<ActionRecord>,
    pub results: Vec<GameResultRecord>,
    pub players: Vec<(Uuid, String)>,
}

impl GameEventCollector {
    pub fn new(metadata: GameMetadata) -> Self {
        Self {
            metadata,
            actions: Vec::new(),
            results: Vec::new(),
            players: Vec::new(),
        }
    }

    pub fn add_action(&mut self, mut action: ActionRecord) {
        // Set game_id to 0 for now - will be updated during save
        action.game_id = 0;
        self.actions.push(action);
    }

    pub fn add_result(&mut self, result: GameResultRecord) {
        self.results.push(result);
    }

    pub fn add_player(&mut self, player_id: Uuid, name: String) {
        self.players.push((player_id, name));
    }

    pub fn actions(&self) -> &[ActionRecord] {
        &self.actions
    }

    pub fn results(&self) -> &[GameResultRecord] {
        &self.results
    }

    pub fn players(&self) -> &[(Uuid, String)] {
        &self.players
    }
}

#[tokio::test]
async fn test_game_event_collection() {
    use crate::collectors::{GameEventCollector, GameMetadata};
    use uuid::Uuid;
    use chrono::Utc;

    let player_order = vec![Uuid::new_v4()];
    let metadata = GameMetadata {
        started_at: Utc::now(),
        num_players: 1,
        deck_seed: "test".to_string(),
        player_order: player_order.clone(),
        configuration: None,
    };

    let mut collector = GameEventCollector::new(metadata);

    // Test action collection
    let action = crate::models::ActionRecord {
        id: None,
        game_id: 1, // Will be set by collector
        player_id: player_order[0],
        action_type: "Pass".to_string(),
        card_play: None,
        target_player_id: None,
        turn_order: 1,
        phase: "test".to_string(),
        created_at: Utc::now(),
    };

    collector.add_action(action);
    assert_eq!(collector.actions().len(), 1);
}
