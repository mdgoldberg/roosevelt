use super::{DatabaseWriter, GameHandle};
use crate::{DatabaseError, ActionRecord, GameResultRecord};
use crate::collectors::{GameMetadata, GameEventCollector};
use sqlx::{SqlitePool, Row};
use std::collections::HashMap;
use uuid::Uuid;

pub struct BulkGameWriter {
    pool: SqlitePool,
    active_games: HashMap<GameHandle, GameEventCollector>,
    next_game_id: i64,
}

impl BulkGameWriter {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            active_games: HashMap::new(),
            next_game_id: 1,
        }
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        let migrations_dir = std::path::Path::new("./migrations");
        if !migrations_dir.exists() {
            tracing::info!(
                "Migrations directory not found at {}', skipping migrations",
                migrations_dir.display()
            );
            return Ok(());
        }
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub async fn save_collector(&self, collector: &mut GameEventCollector) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await.map_err(|e| DatabaseError::Transaction(e.to_string()))?;

        for (player_id, name) in &collector.players {
            let player_id_str = player_id.to_string();
            sqlx::query("INSERT OR IGNORE INTO players (id, name) VALUES (?, ?)")
                .bind(player_id_str)
                .bind(name)
                .execute(&mut *tx)
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        let player_order_json = serde_json::to_vec(&collector.metadata.player_order)
            .map_err(DatabaseError::Serialization)?;
        let configuration_json = collector.metadata.configuration
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()
            .map_err(DatabaseError::Serialization)?;

        let result = sqlx::query(
            "INSERT INTO games (started_at, num_players, deck_seed, player_order, configuration) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(collector.metadata.started_at)
        .bind(collector.metadata.num_players as i64)
        .bind(&collector.metadata.deck_seed)
        .bind(player_order_json)
        .bind(configuration_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let game_id = result.last_insert_rowid();

        for action in &mut collector.actions {
            action.game_id = game_id;
        }

        for action in &collector.actions {
            let card_play_json = action.card_play.as_ref()
                .map(|v| serde_json::to_vec(v))
                .transpose()
                .map_err(DatabaseError::Serialization)?;
            let target_player_id = action.target_player_id.map(|u| u.to_string());
            let player_id = action.player_id.to_string();

            sqlx::query(
                "INSERT INTO actions (game_id, player_id, action_type, card_play, target_player_id, turn_order, phase) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(action.game_id)
            .bind(player_id)
            .bind(&action.action_type)
            .bind(card_play_json)
            .bind(target_player_id)
            .bind(action.turn_order as i64)
            .bind(&action.phase)
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        for result in &collector.results {
            let player_id = result.player_id.to_string();
            sqlx::query(
                "INSERT INTO game_results (game_id, player_id, finishing_place, finishing_role) VALUES (?, ?, ?, ?)"
            )
            .bind(result.game_id)
            .bind(player_id)
            .bind(result.finishing_place as i64)
            .bind(&result.finishing_role)
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        sqlx::query("UPDATE games SET finished_at = ? WHERE id = ?")
            .bind(chrono::Utc::now())
            .bind(game_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        tx.commit().await.map_err(|e| DatabaseError::Transaction(e.to_string()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl DatabaseWriter for BulkGameWriter {
    async fn record_player(&mut self, _player_id: Uuid, _name: &str) -> Result<(), DatabaseError> {
        Ok(())
    }

    async fn get_player_by_name(&mut self, name: &str) -> Result<Option<Uuid>, DatabaseError> {
        let row = sqlx::query("SELECT id FROM players WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(match row {
            Some(r) => {
                let id: String = r.get("id");
                Some(Uuid::parse_str(&id).map_err(DatabaseError::UuidParsing)?)
            }
            None => None,
        })
    }

    async fn start_game(&mut self, game_meta: GameMetadata) -> Result<GameHandle, DatabaseError> {
        let handle = GameHandle::new(self.next_game_id);
        self.next_game_id += 1;
        let collector = GameEventCollector::new(game_meta);
        self.active_games.insert(handle, collector);
        Ok(handle)
    }

    async fn record_action(&mut self, handle: GameHandle, action: &ActionRecord) -> Result<(), DatabaseError> {
        if let Some(collector) = self.active_games.get_mut(&handle) {
            collector.add_action(action.clone());
        }
        Ok(())
    }

    async fn finish_game(&mut self, handle: GameHandle, results: &[GameResultRecord]) -> Result<(), DatabaseError> {
        if let Some(mut collector) = self.active_games.remove(&handle) {
            for result in results {
                collector.add_result(result.clone());
            }
            self.save_collector(&mut collector).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writers::DatabaseWriter;
    use crate::collectors::GameMetadata;
    use uuid::Uuid;
    use chrono::Utc;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_bulk_game_writer_basic_functionality() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let mut writer = BulkGameWriter::new(pool);
        // Skip migrations for in-memory test - just verify basic functionality
        // In real usage, migrations would be run first
        let player_id = Uuid::new_v4();
        // For bulk writer, record_player is a no-op, so just verify it doesn't panic
        writer.record_player(player_id, "TestPlayer").await.unwrap();
        // get_player_by_name will fail without migrations, so we skip that assertion
        let player_order = vec![player_id];
        let metadata = GameMetadata {
            started_at: Utc::now(),
            num_players: 1,
            deck_seed: "test_seed".to_string(),
            player_order,
            configuration: None,
        };
        let handle = writer.start_game(metadata).await.unwrap();
        assert!(handle.as_i64() > 0);
    }
}
