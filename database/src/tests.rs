#[cfg(test)]
mod database_tests {
    use crate::*;

    pub async fn setup_test_db() -> sqlx::sqlite::SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database pool");

        sqlx::query(
            r#"
            CREATE TABLE players (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                finished_at TIMESTAMP,
                num_players INTEGER NOT NULL,
                deck_seed TEXT NOT NULL,
                player_order JSON NOT NULL,
                configuration JSON
            );

            CREATE TABLE game_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL REFERENCES games(id),
                player_id TEXT NOT NULL REFERENCES players(id),
                finishing_place INTEGER NOT NULL,
                finishing_role TEXT NOT NULL,
                UNIQUE(game_id, player_id)
            );

            CREATE TABLE actions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL REFERENCES games(id),
                player_id TEXT NOT NULL REFERENCES players(id),
                action_type TEXT NOT NULL,
                card_play JSON,
                target_player_id TEXT REFERENCES players(id),
                turn_order INTEGER NOT NULL,
                phase TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE failed_writes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                error_type TEXT NOT NULL,
                error_message TEXT NOT NULL,
                data TEXT
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to run test migrations");

        pool
    }

    #[tokio::test]
    async fn test_record_and_retrieve_player() {
        let pool = setup_test_db().await;
        let recorder: DatabaseRecorder = DatabaseRecorder::new(pool);

        let player_id = uuid::Uuid::new_v4();
        let player_name = "Test Player";

        recorder
            .record_player(player_id, player_name)
            .await
            .expect("Failed to record player");

        let retrieved_id = recorder
            .get_player_by_name(player_name)
            .await
            .expect("Failed to retrieve player");

        assert_eq!(retrieved_id, Some(player_id));
    }

    #[tokio::test]
    async fn test_record_game() {
        let pool = setup_test_db().await;
        let recorder: DatabaseRecorder = DatabaseRecorder::new(pool);

        let player_id = uuid::Uuid::new_v4();
        recorder
            .record_player(player_id, "Test Player")
            .await
            .expect("Failed to record player");

        let game_record = GameRecord {
            id: None,
            started_at: chrono::Utc::now(),
            finished_at: None,
            num_players: 1,
            deck_seed: "test_seed".to_string(),
            player_order: vec![player_id],
            configuration: None,
        };

        let game_id = recorder
            .record_game(&game_record)
            .await
            .expect("Failed to record game");

        assert!(game_id > 0);
    }

    #[tokio::test]
    async fn test_record_action() {
        let pool = setup_test_db().await;
        let recorder: DatabaseRecorder = DatabaseRecorder::new(pool);

        let player_id = uuid::Uuid::new_v4();
        recorder
            .record_player(player_id, "Test Player")
            .await
            .expect("Failed to record player");

        let game_record = GameRecord {
            id: None,
            started_at: chrono::Utc::now(),
            finished_at: None,
            num_players: 1,
            deck_seed: "test_seed".to_string(),
            player_order: vec![player_id],
            configuration: None,
        };

        let game_id = recorder
            .record_game(&game_record)
            .await
            .expect("Failed to record game");

        let action_record = ActionRecord {
            id: None,
            game_id,
            player_id,
            action_type: "PlayCards".to_string(),
            card_play: Some(serde_json::json!({
                "type": "Single",
                "cards": ["3♠"]
            })),
            target_player_id: None,
            turn_order: 1,
            phase: "ingame".to_string(),
            created_at: chrono::Utc::now(),
        };

        recorder
            .record_action(&action_record)
            .await
            .expect("Failed to record action");
    }

    #[tokio::test]
    async fn test_record_game_result() {
        let pool = setup_test_db().await;
        let recorder: DatabaseRecorder = DatabaseRecorder::new(pool);

        let player_id = uuid::Uuid::new_v4();
        recorder
            .record_player(player_id, "Test Player")
            .await
            .expect("Failed to record player");

        let game_record = GameRecord {
            id: None,
            started_at: chrono::Utc::now(),
            finished_at: None,
            num_players: 1,
            deck_seed: "test_seed".to_string(),
            player_order: vec![player_id],
            configuration: None,
        };

        let game_id = recorder
            .record_game(&game_record)
            .await
            .expect("Failed to record game");

        let game_result = GameResultRecord {
            id: None,
            game_id,
            player_id,
            finishing_place: 1,
            finishing_role: "President".to_string(),
        };

        recorder
            .record_game_result(&game_result)
            .await
            .expect("Failed to record game result");
    }

    #[tokio::test]
    async fn test_finish_game() {
        let pool = setup_test_db().await;
        let recorder: DatabaseRecorder = DatabaseRecorder::new(pool);

        let player_id = uuid::Uuid::new_v4();
        recorder
            .record_player(player_id, "Test Player")
            .await
            .expect("Failed to record player");

        let game_record = GameRecord {
            id: None,
            started_at: chrono::Utc::now(),
            finished_at: None,
            num_players: 1,
            deck_seed: "test_seed".to_string(),
            player_order: vec![player_id],
            configuration: None,
        };

        let game_id = recorder
            .record_game(&game_record)
            .await
            .expect("Failed to record game");

        let finished_at = chrono::Utc::now();

        recorder
            .finish_game(game_id, finished_at)
            .await
            .expect("Failed to finish game");
    }

    #[tokio::test]
    async fn test_noop_recorder() {
        let recorder = NoopRecorder;

        let player_id = uuid::Uuid::new_v4();

        recorder
            .record_player(player_id, "Test Player")
            .await
            .expect("NoopRecorder should always succeed");

        let retrieved = recorder
            .get_player_by_name("Test Player")
            .await
            .expect("NoopRecorder should always succeed");

        assert_eq!(retrieved, None);

        let game_record = GameRecord {
            id: None,
            started_at: chrono::Utc::now(),
            finished_at: None,
            num_players: 1,
            deck_seed: "test_seed".to_string(),
            player_order: vec![player_id],
            configuration: None,
        };

        let game_id = recorder
            .record_game(&game_record)
            .await
            .expect("NoopRecorder should always succeed");

        assert_eq!(game_id, 0);

        let action_record = ActionRecord {
            id: None,
            game_id,
            player_id,
            action_type: "PlayCards".to_string(),
            card_play: None,
            target_player_id: None,
            turn_order: 1,
            phase: "ingame".to_string(),
            created_at: chrono::Utc::now(),
        };

        recorder
            .record_action(&action_record)
            .await
            .expect("NoopRecorder should always succeed");

        let game_result = GameResultRecord {
            id: None,
            game_id,
            player_id,
            finishing_place: 1,
            finishing_role: "President".to_string(),
        };

        recorder
            .record_game_result(&game_result)
            .await
            .expect("NoopRecorder should always succeed");

        recorder
            .finish_game(game_id, chrono::Utc::now())
            .await
            .expect("NoopRecorder should always succeed");
    }
}
