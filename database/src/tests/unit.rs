#[cfg(test)]
mod tests {
    mod writer_tests {
        use crate::writers::DatabaseWriter;
        use uuid::Uuid;

        #[tokio::test]
        async fn test_database_writer_trait_compiles() {
            // This test ensures that trait is properly defined
            // We'll implement mock writers in later tasks
            fn _check_trait_bounds<W: DatabaseWriter>(_writer: W) {}

            // If this compiles, trait is defined correctly
            assert!(true);
        }
    }

    mod collector_tests {
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
    }
}
