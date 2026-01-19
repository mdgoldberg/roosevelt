pub mod config;
pub mod error;
pub mod models;
pub mod repository;
pub mod retry;

pub use config::DatabaseConfig;
pub use error::DatabaseError;
pub use models::{ActionRecord, FailedWrite, GameRecord, GameResultRecord, PlayerRecord};
pub use repository::DatabaseRecorder;
pub use retry::retry_with_backoff;

#[async_trait::async_trait]
pub trait GameRecorder: Send + Sync {
    async fn record_player(
        &self,
        player_id: uuid::Uuid,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_player_by_name(
        &self,
        name: &str,
    ) -> Result<Option<uuid::Uuid>, Box<dyn std::error::Error>>;
    async fn record_game(&self, game: &GameRecord) -> Result<i64, Box<dyn std::error::Error>>;
    async fn record_action(&self, action: &ActionRecord) -> Result<(), Box<dyn std::error::Error>>;
    async fn record_game_result(
        &self,
        result: &GameResultRecord,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn finish_game(
        &self,
        game_id: i64,
        finished_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct NoopRecorder;

#[async_trait::async_trait]
impl GameRecorder for NoopRecorder {
    async fn record_player(
        &self,
        _player_id: uuid::Uuid,
        _name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn get_player_by_name(
        &self,
        _name: &str,
    ) -> Result<Option<uuid::Uuid>, Box<dyn std::error::Error>> {
        Ok(None)
    }

    async fn record_game(&self, _game: &GameRecord) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(0)
    }

    async fn record_action(
        &self,
        _action: &ActionRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn record_game_result(
        &self,
        _result: &GameResultRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn finish_game(
        &self,
        _game_id: i64,
        _finished_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
