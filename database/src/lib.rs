pub mod collectors;
pub mod config;
pub mod error;
pub mod models;
pub mod retry;
pub mod writers;

pub use collectors::{GameEventCollector, GameMetadata};
pub use config::DatabaseConfig;
pub use error::DatabaseError;
pub use models::{ActionRecord, FailedWrite, GameRecord, GameResultRecord, PlayerRecord};
pub use retry::retry_with_backoff;
pub use writers::{BulkGameWriter, DatabaseWriter, GameHandle, StreamingGameWriter};

// NoopRecorder for when database persistence is not needed
pub struct NoopRecorder;

#[async_trait::async_trait]
impl writers::DatabaseWriter for NoopRecorder {
    async fn record_player(
        &mut self,
        _player_id: uuid::Uuid,
        _name: &str,
    ) -> Result<(), error::DatabaseError> {
        Ok(())
    }

    async fn get_player_by_name(
        &mut self,
        _name: &str,
    ) -> Result<Option<uuid::Uuid>, error::DatabaseError> {
        Ok(None)
    }

    async fn start_game(
        &mut self,
        _game_meta: collectors::GameMetadata,
    ) -> Result<writers::GameHandle, error::DatabaseError> {
        Ok(writers::GameHandle::new(0))
    }

    async fn record_action(
        &mut self,
        _handle: writers::GameHandle,
        _action: &models::ActionRecord,
    ) -> Result<(), error::DatabaseError> {
        Ok(())
    }

    async fn finish_game(
        &mut self,
        _handle: writers::GameHandle,
        _results: &[models::GameResultRecord],
    ) -> Result<(), error::DatabaseError> {
        Ok(())
    }
}
