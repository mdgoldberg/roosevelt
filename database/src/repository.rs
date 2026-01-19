use super::models::*;
use super::GameRecorder;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub struct DatabaseRecorder {
    pool: SqlitePool,
}

impl DatabaseRecorder {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl GameRecorder for DatabaseRecorder {
    async fn record_player(
        &self,
        player_id: Uuid,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let player_id_str = player_id.to_string();
        sqlx::query("INSERT INTO players (id, name) VALUES (?, ?)")
            .bind(player_id_str)
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_player_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
        let row = sqlx::query("SELECT id FROM players WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| {
            let id: String = r.get("id");
            Uuid::parse_str(&id).unwrap()
        }))
    }

    async fn record_game(&self, game: &GameRecord) -> Result<i64, Box<dyn std::error::Error>> {
        let player_order_json = serde_json::to_vec(&game.player_order)?;
        let configuration_json = game
            .configuration
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()?;

        let result = sqlx::query(
            "INSERT INTO games (started_at, num_players, deck_seed, player_order, configuration)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(game.started_at)
        .bind(game.num_players as i64)
        .bind(&game.deck_seed)
        .bind(player_order_json)
        .bind(configuration_json)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn record_action(&self, action: &ActionRecord) -> Result<(), Box<dyn std::error::Error>> {
        let card_play_json = action
            .card_play
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()?;
        let target_player_id = action.target_player_id.map(|u| u.to_string());
        let player_id = action.player_id.to_string();

        sqlx::query(
            "INSERT INTO actions (game_id, player_id, action_type, card_play, target_player_id, turn_order, phase)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(action.game_id)
        .bind(player_id)
        .bind(&action.action_type)
        .bind(card_play_json)
        .bind(target_player_id)
        .bind(action.turn_order as i64)
        .bind(&action.phase)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn record_game_result(
        &self,
        result: &GameResultRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let player_id = result.player_id.to_string();

        sqlx::query(
            "INSERT INTO game_results (game_id, player_id, finishing_place, finishing_role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(result.game_id)
        .bind(player_id)
        .bind(result.finishing_place as i64)
        .bind(&result.finishing_role)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn finish_game(
        &self,
        game_id: i64,
        finished_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("UPDATE games SET finished_at = ? WHERE id = ?")
            .bind(finished_at)
            .bind(game_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
