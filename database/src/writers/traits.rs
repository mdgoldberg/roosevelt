use super::super::collectors::GameMetadata;
use super::super::{ActionRecord, DatabaseError, GameResultRecord};
use super::game_handle::GameHandle;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait DatabaseWriter: Send + Sync {
    async fn record_player(&mut self, player_id: Uuid, name: &str) -> Result<(), DatabaseError>;
    async fn get_player_by_name(&mut self, name: &str) -> Result<Option<Uuid>, DatabaseError>;
    async fn start_game(&mut self, game_meta: GameMetadata) -> Result<GameHandle, DatabaseError>;
    async fn record_action(
        &mut self,
        handle: GameHandle,
        action: &ActionRecord,
    ) -> Result<(), DatabaseError>;
    async fn finish_game(
        &mut self,
        handle: GameHandle,
        results: &[GameResultRecord],
    ) -> Result<(), DatabaseError>;
}
