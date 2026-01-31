# Transactional Database Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace individual database transactions with bulk game recording using a flexible DatabaseWriter trait that supports both bulk and streaming implementations.

**Architecture:** Implement a DatabaseWriter trait with BulkGameWriter (for simulation) and StreamingGameWriter (for web apps) that collect game events and persist them atomically with retry logic.

**Tech Stack:** Rust async with SQLx, tokio, uuid, chrono, serde_json

---

### Task 1: Create basic types and trait definitions

**Files:**
- Create: `database/src/collectors/mod.rs`
- Create: `database/src/collectors/game_metadata.rs`
- Create: `database/src/writers/mod.rs`
- Create: `database/src/writers/traits.rs`
- Create: `database/src/writers/game_handle.rs`
- Create: `database/src/tests/unit/collector_tests.rs`

**Step 1: Write the failing test for GameMetadata**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p database collector_tests::test_game_metadata_creation`
Expected: FAIL with "GameMetadata not defined"

**Step 3: Write minimal GameMetadata implementation**

In `database/src/collectors/game_metadata.rs`:

```rust
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p database collector_tests::test_game_metadata_creation`
Expected: PASS

**Step 5: Commit**

```bash
git add database/src/collectors/
git commit -m "feat: add GameMetadata struct for game configuration"
```

### Task 2: Implement DatabaseWriter trait and GameHandle

**Files:**
- Modify: `database/src/writers/traits.rs`
- Modify: `database/src/writers/game_handle.rs`
- Modify: `database/src/tests/unit/writer_tests.rs`

**Step 1: Write the failing test for DatabaseWriter trait**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::writers::DatabaseWriter;
    use crate::collectors::GameMetadata;
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p database writer_tests::test_database_writer_trait_compiles`
Expected: FAIL with "DatabaseWriter not found in crate"

**Step 3: Write minimal trait implementation**

In `database/src/writers/traits.rs`:

```rust
use super::super::{DatabaseError, ActionRecord, GameResultRecord};
use super::game_handle::GameHandle;
use super::super::collectors::GameMetadata;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait DatabaseWriter: Send + Sync {
    async fn record_player(&mut self, player_id: Uuid, name: &str) -> Result<(), DatabaseError>;
    async fn get_player_by_name(&mut self, name: &str) -> Result<Option<Uuid>, DatabaseError>;
    async fn start_game(&mut self, game_meta: GameMetadata) -> Result<GameHandle, DatabaseError>;
    async fn record_action(&mut self, handle: GameHandle, action: &ActionRecord) -> Result<(), DatabaseError>;
    async fn finish_game(&mut self, handle: GameHandle, results: &[GameResultRecord]) -> Result<(), DatabaseError>;
}
```

In `database/src/writers/game_handle.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameHandle(pub(crate) i64);

impl GameHandle {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p database writer_tests::test_database_writer_trait_compiles`
Expected: PASS

**Step 5: Commit**

```bash
git add database/src/writers/
git commit -m "feat: add DatabaseWriter trait and GameHandle type"
```

### Task 3: Implement GameEventCollector for bulk data collection

**Files:**
- Create: `database/src/collectors/game_collector.rs`
- Modify: `database/src/tests/unit/collector_tests.rs`

**Step 1: Write the failing test for GameEventCollector**

```rust
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
    let action = ActionRecord {
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p database collector_tests::test_game_event_collection`
Expected: FAIL with "GameEventCollector not defined"

**Step 3: Write minimal GameEventCollector implementation**

In `database/src/collectors/game_collector.rs`:

```rust
use super::{GameMetadata, ActionRecord, GameResultRecord};
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p database collector_tests::test_game_event_collection`
Expected: PASS

**Step 5: Commit**

```bash
git add database/src/collectors/game_collector.rs
git commit -m "feat: implement GameEventCollector for bulk data collection"
```

### Task 4: Implement BulkGameWriter with single transaction logic

**Files:**
- Create: `database/src/writers/bulk_writer.rs`
- Create: `database/src/tests/unit/bulk_writer_tests.rs`

**Step 1: Write the failing test for BulkGameWriter**

```rust
#[tokio::test]
async fn test_bulk_game_writer_basic_functionality() {
    use crate::writers::{BulkGameWriter, DatabaseWriter};
    use crate::collectors::{GameEventCollector, GameMetadata};
    use uuid::Uuid;
    use chrono::Utc;
    use sqlx::SqlitePool;

    // Use in-memory database for testing
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    let mut writer = BulkGameWriter::new(pool);

    // Test player operations
    let player_id = Uuid::new_v4();
    writer.record_player(player_id, "TestPlayer").await.unwrap();

    let found_id = writer.get_player_by_name("TestPlayer").await.unwrap();
    assert_eq!(found_id, Some(player_id));

    // Test game operations
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p database bulk_writer_tests::test_bulk_game_writer_basic_functionality`
Expected: FAIL with "BulkGameWriter not defined"

**Step 3: Write minimal BulkGameWriter implementation**

In `database/src/writers/bulk_writer.rs`:

```rust
use super::{DatabaseWriter, GameHandle};
use super::super::{DatabaseError, ActionRecord, GameResultRecord};
use super::super::collectors::{GameMetadata, GameEventCollector};
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
        // Pre-flight check for migration directory
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

    pub async fn save_collector(&mut self, mut collector: GameEventCollector) -> Result<(), DatabaseError> {
        // Use retry logic for the entire transaction
        crate::retry_with_backoff(
            || Box::pin(self.save_collector_internal(&mut collector)),
            5,
            std::time::Duration::from_millis(100),
        ).await.map_err(|e| DatabaseError::RetryExhausted(e.to_string()))
    }

    async fn save_collector_internal(&mut self, collector: &mut GameEventCollector) -> Result<(), Box<dyn std::error::Error>> {
        let mut tx = self.pool.begin().await.map_err(|e| DatabaseError::Transaction(e.to_string()))?;

        // Insert players
        for (player_id, name) in &collector.players {
            let player_id_str = player_id.to_string();
            sqlx::query("INSERT OR IGNORE INTO players (id, name) VALUES (?, ?)")
                .bind(player_id_str)
                .bind(name)
                .execute(&mut *tx)
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        // Insert game and get ID
        let player_order_json = serde_json::to_vec(&collector.metadata.player_order)
            .map_err(DatabaseError::Serialization)?;
        let configuration_json = collector.metadata.configuration
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()
            .map_err(DatabaseError::Serialization)?;

        let result = sqlx::query(
            "INSERT INTO games (started_at, num_players, deck_seed, player_order, configuration)
             VALUES (?, ?, ?, ?, ?)"
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

        // Update actions with game_id
        for action in &mut collector.actions {
            action.game_id = game_id;
        }

        // Insert actions
        for action in &collector.actions {
            let card_play_json = action.card_play.as_ref()
                .map(|v| serde_json::to_vec(v))
                .transpose()
                .map_err(DatabaseError::Serialization)?;
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
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        // Insert results
        for result in &collector.results {
            let player_id = result.player_id.to_string();
            sqlx::query(
                "INSERT INTO game_results (game_id, player_id, finishing_place, finishing_role)
                 VALUES (?, ?, ?, ?)"
            )
            .bind(result.game_id)
            .bind(player_id)
            .bind(result.finishing_place as i64)
            .bind(&result.finishing_role)
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        // Update game as finished
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

#[async_trait]
impl DatabaseWriter for BulkGameWriter {
    async fn record_player(&mut self, player_id: Uuid, name: &str) -> Result<(), DatabaseError> {
        // For bulk writer, we don't insert immediately - just record for later
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

    async fn record_action(&mut self, handle: GameHandle, action: ActionRecord) -> Result<(), DatabaseError> {
        if let Some(collector) = self.active_games.get_mut(&handle) {
            collector.add_action(action);
        }
        Ok(())
    }

    async fn finish_game(&mut self, handle: GameHandle, results: &[GameResultRecord]) -> Result<(), DatabaseError> {
        if let Some(mut collector) = self.active_games.remove(&handle) {
            for result in results {
                collector.add_result(result.clone());
            }
            self.save_collector(collector).await?;
        }
        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p database bulk_writer_tests::test_bulk_game_writer_basic_functionality`
Expected: PASS

**Step 5: Commit**

```bash
git add database/src/writers/bulk_writer.rs database/src/tests/unit/bulk_writer_tests.rs
git commit -m "feat: implement BulkGameWriter with single transaction logic"
```

### Task 5: Implement StreamingGameWriter for real-time persistence

**Files:**
- Create: `database/src/writers/streaming_writer.rs`
- Create: `database/src/tests/unit/streaming_writer_tests.rs`

**Step 1: Write the failing test for StreamingGameWriter**

```rust
#[tokio::test]
async fn test_streaming_game_writer_persists_immediately() {
    use crate::writers::{StreamingGameWriter, DatabaseWriter};
    use crate::collectors::GameMetadata;
    use uuid::Uuid;
    use chrono::Utc;
    use sqlx::SqlitePool;

    let pool = SqlitePool::connect(":memory:").await.unwrap();
    let mut writer = StreamingGameWriter::new(pool);

    // Record player
    let player_id = Uuid::new_v4();
    writer.record_player(player_id, "TestPlayer").await.unwrap();

    // Start game
    let metadata = GameMetadata {
        started_at: Utc::now(),
        num_players: 1,
        deck_seed: "test".to_string(),
        player_order: vec![player_id],
        configuration: None,
    };
    let handle = writer.start_game(metadata).await.unwrap();

    // Record action
    let action = ActionRecord {
        id: None,
        game_id: handle.as_i64(),
        player_id,
        action_type: "Pass".to_string(),
        card_play: None,
        target_player_id: None,
        turn_order: 1,
        phase: "test".to_string(),
        created_at: Utc::now(),
    };

    writer.record_action(handle, action).await.unwrap();

    // Verify action is immediately persisted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM actions")
        .fetch_one(&writer.pool)
        .await.unwrap();
    assert_eq!(count, 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p database streaming_writer_tests::test_streaming_game_writer_persists_immediately`
Expected: FAIL with "StreamingGameWriter not defined"

**Step 3: Write minimal StreamingGameWriter implementation**

In `database/src/writers/streaming_writer.rs`:

```rust
use super::{DatabaseWriter, GameHandle};
use super::super::{DatabaseError, ActionRecord, GameResultRecord};
use super::super::collectors::GameMetadata;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

pub struct StreamingGameWriter {
    pool: SqlitePool,
    next_game_id: i64,
}

impl StreamingGameWriter {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            next_game_id: 1,
        }
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Pre-flight check for migration directory
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

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl DatabaseWriter for StreamingGameWriter {
    async fn record_player(&mut self, player_id: Uuid, name: &str) -> Result<(), DatabaseError> {
        let player_id_str = player_id.to_string();
        sqlx::query("INSERT OR IGNORE INTO players (id, name) VALUES (?, ?)")
            .bind(player_id_str)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
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
        crate::retry_with_backoff(
            || Box::pin(self.insert_game(&game_meta)),
            5,
            std::time::Duration::from_millis(100),
        ).await.map_err(|e| DatabaseError::RetryExhausted(e.to_string()))
    }

    async fn record_action(&mut self, handle: GameHandle, action: ActionRecord) -> Result<(), DatabaseError> {
        crate::retry_with_backoff(
            || Box::pin(self.insert_action(&handle, &action)),
            5,
            std::time::Duration::from_millis(100),
        ).await.map_err(|e| DatabaseError::RetryExhausted(e.to_string()))
    }

    async fn finish_game(&mut self, handle: GameHandle, results: &[GameResultRecord]) -> Result<(), DatabaseError> {
        crate::retry_with_backoff(
            || Box::pin(self.finish_game_internal(&handle, results)),
            5,
            std::time::Duration::from_millis(100),
        ).await.map_err(|e| DatabaseError::RetryExhausted(e.to_string()))
    }
}

impl StreamingGameWriter {
    async fn insert_game(&mut self, game_meta: &GameMetadata) -> Result<GameHandle, Box<dyn std::error::Error>> {
        let player_order_json = serde_json::to_vec(&game_meta.player_order)?;
        let configuration_json = game_meta.configuration
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()?;

        let result = sqlx::query(
            "INSERT INTO games (started_at, num_players, deck_seed, player_order, configuration)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(game_meta.started_at)
        .bind(game_meta.num_players as i64)
        .bind(&game_meta.deck_seed)
        .bind(player_order_json)
        .bind(configuration_json)
        .execute(&self.pool)
        .await?;

        let game_id = result.last_insert_rowid();
        Ok(GameHandle::new(game_id))
    }

    async fn insert_action(&self, handle: &GameHandle, action: &ActionRecord) -> Result<(), Box<dyn std::error::Error>> {
        let card_play_json = action.card_play.as_ref()
            .map(|v| serde_json::to_vec(v))
            .transpose()?;
        let target_player_id = action.target_player_id.map(|u| u.to_string());
        let player_id = action.player_id.to_string();

        sqlx::query(
            "INSERT INTO actions (game_id, player_id, action_type, card_play, target_player_id, turn_order, phase)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(handle.as_i64())
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

    async fn finish_game_internal(&self, handle: &GameHandle, results: &[GameResultRecord]) -> Result<(), Box<dyn std::error::Error>> {
        let mut tx = self.pool.begin().await?;

        // Insert results
        for result in results {
            let player_id = result.player_id.to_string();
            sqlx::query(
                "INSERT INTO game_results (game_id, player_id, finishing_place, finishing_role)
                 VALUES (?, ?, ?, ?)"
            )
            .bind(handle.as_i64())
            .bind(player_id)
            .bind(result.finishing_place as i64)
            .bind(&result.finishing_role)
            .execute(&mut *tx)
            .await?;
        }

        // Update game as finished
        sqlx::query("UPDATE games SET finished_at = ? WHERE id = ?")
            .bind(chrono::Utc::now())
            .bind(handle.as_i64())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p database streaming_writer_tests::test_streaming_game_writer_persists_immediately`
Expected: PASS

**Step 5: Commit**

```bash
git add database/src/writers/streaming_writer.rs database/src/tests/unit/streaming_writer_tests.rs
git commit -m "feat: implement StreamingGameWriter for real-time persistence"
```

### Task 6: Update lib.rs exports and module structure

**Files:**
- Modify: `database/src/lib.rs`
- Modify: `database/src/collectors/mod.rs`
- Modify: `database/src/writers/mod.rs`

**Step 1: Update collector module exports**

In `database/src/collectors/mod.rs`:

```rust
pub mod game_collector;
pub mod game_metadata;

pub use game_collector::GameEventCollector;
pub use game_metadata::GameMetadata;
```

**Step 2: Update writer module exports**

In `database/src/writers/mod.rs`:

```rust
pub mod bulk_writer;
pub mod streaming_writer;
pub mod traits;
pub mod game_handle;

pub use bulk_writer::BulkGameWriter;
pub use streaming_writer::StreamingGameWriter;
pub use traits::DatabaseWriter;
pub use game_handle::GameHandle;
```

**Step 3: Update main library exports**

In `database/src/lib.rs`:

```rust
pub mod config;
pub mod error;
pub mod models;
pub mod retry;
pub mod writers;
pub mod collectors;

pub use config::DatabaseConfig;
pub use error::DatabaseError;
pub use models::{ActionRecord, FailedWrite, GameRecord, GameResultRecord, PlayerRecord};
pub use retry::retry_with_backoff;
pub use writers::{DatabaseWriter, BulkGameWriter, StreamingGameWriter, GameHandle};
pub use collectors::{GameEventCollector, GameMetadata};

// Keep NoopRecorder for backward compatibility during transition
pub struct NoopRecorder;

#[async_trait::async_trait]
impl crate::writers::DatabaseWriter for NoopRecorder {
    async fn record_player(
        &mut self,
        _player_id: uuid::Uuid,
        _name: &str,
    ) -> Result<(), crate::DatabaseError> {
        Ok(())
    }

    async fn get_player_by_name(
        &mut self,
        _name: &str,
    ) -> Result<Option<uuid::Uuid>, crate::DatabaseError> {
        Ok(None)
    }

    async fn start_game(
        &mut self,
        _game_meta: crate::collectors::GameMetadata,
    ) -> Result<crate::writers::GameHandle, crate::DatabaseError> {
        Ok(crate::writers::GameHandle::new(0))
    }

    async fn record_action(
        &mut self,
        _handle: crate::writers::GameHandle,
        _action: crate::models::ActionRecord,
    ) -> Result<(), crate::DatabaseError> {
        Ok(())
    }

    async fn finish_game(
        &mut self,
        _handle: crate::writers::GameHandle,
        _results: &[crate::models::GameResultRecord],
    ) -> Result<(), crate::DatabaseError> {
        Ok(())
    }
}
```

**Step 4: Run tests to verify module structure**

Run: `cargo test -p database`
Expected: All tests pass

**Step 5: Commit**

```bash
git add database/src/lib.rs database/src/collectors/mod.rs database/src/writers/mod.rs
git commit -m "refactor: update library exports for new DatabaseWriter architecture"
```

### Task 7: Update simulation to use new DatabaseWriter interface

**Files:**
- Modify: `simulation/src/lib.rs`
- Test: `simulation/src/tests/integration_tests.rs`

**Step 1: Write the failing test for updated simulation**

```rust
#[tokio::test]
async fn test_simulation_uses_new_database_writer() {
    use simulation::run_game;
    use types::game_state::GameState;
    use types::{Player, Strategy, DefaultStrategy};
    use database::{BulkGameWriter, DatabaseWriter};
    use sqlx::SqlitePool;
    use uuid::Uuid;

    let pool = SqlitePool::connect(":memory:").await.unwrap();
    let mut writer = BulkGameWriter::new(pool);
    writer.run_migrations().await.unwrap();

    // Create simple game state
    let players = vec![
        Player {
            state: types::PlayerState::new(Uuid::new_v4(), "Player1"),
            strategy: Box::new(DefaultStrategy {}),
        },
        Player {
            state: types::PlayerState::new(Uuid::new_v4(), "Player2"),
            strategy: Box::new(DefaultStrategy {}),
        },
    ];

    let mut game_state = GameState::new(players, 12345);

    // This should use new DatabaseWriter interface
    let result = run_game(&mut game_state, None, &mut writer, None).await;
    assert!(result.is_ok());

    // Verify game was recorded completely
    let game_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM games")
        .fetch_one(&writer.pool)
        .await.unwrap();
    assert_eq!(game_count, 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p simulation integration_tests::test_simulation_uses_new_database_writer`
Expected: FAIL with compile errors due to interface mismatch

**Step 3: Update simulation to use new DatabaseWriter**

In `simulation/src/lib.rs`, replace entire file content:

```rust
use std::{thread::sleep, time::Duration};
use types::game_state::GameState;
use types::{Action, Player};
use database::{DatabaseWriter, GameMetadata};

pub async fn run_game(
    game_state: &mut GameState,
    delay_ms: Option<u64>,
    recorder: &mut dyn DatabaseWriter,
    game_config: Option<serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let seed = generate_deck_seed();
    let player_order = game_state.table.iter().map(|p| p.state.id).collect();

    // Record players first
    for player in &game_state.table {
        recorder.record_player(player.state.id, &format!("Player_{}", player.state.id)).await?;
    }

    let game_meta = GameMetadata {
        started_at: chrono::Utc::now(),
        num_players: game_state.table.len(),
        deck_seed: format!("{:x}", seed),
        player_order,
        configuration: game_config,
    };

    let handle = recorder.start_game(game_meta).await?;

    let pregame_events = game_state.run_pregame();

    let mut turn_order = 0;
    for event in &pregame_events {
        let (action_type, card_play_json, target_player_id) = match &event.action {
            Action::SendCard { card, to } => (
                "SendCard".to_string(),
                Some(serialize_card_play_single(card)),
                Some(to),
            ),
            _ => (action_type_to_string(&event.action), None, None),
        };

        recorder.record_action(handle, database::models::ActionRecord {
            id: None,
            game_id: handle.as_i64(),
            player_id: event.player_id,
            action_type,
            card_play: card_play_json.map(|v| serde_json::to_value(&v).unwrap()),
            target_player_id: target_player_id.copied(),
            turn_order: turn_order + 1,
            phase: "pregame".to_string(),
            created_at: chrono::Utc::now(),
        }).await?;

        turn_order += 1;
    }

    while game_state.still_playing() {
        log::debug!("{game_state}");
        if let Some(ms) = delay_ms {
            sleep(Duration::from_millis(ms));
        }
        let available_actions = game_state.permitted_actions();
        let public_info = game_state.public_info();
        let current_player = game_state.current_player_mut();
        let selected_action = current_player.strategy.select_action(
            &current_player.state,
            &public_info,
            &available_actions,
        );

        let (action_type, card_play_json, target_player_id) = match &selected_action {
            Action::PlayCards { card_play } => (
                "PlayCards".to_string(),
                Some(serialize_card_play(card_play)),
                None,
            ),
            Action::SendCard { card, to } => (
                "SendCard".to_string(),
                Some(serialize_card_play_single(card)),
                Some(to),
            ),
            Action::Pass => ("Pass".to_string(), None, None),
        };

        recorder.record_action(handle, database::models::ActionRecord {
            id: None,
            game_id: handle.as_i64(),
            player_id: current_player.state.id,
            action_type,
            card_play: card_play_json.map(|v| serde_json::to_value(&v).unwrap()),
            target_player_id: target_player_id.copied(),
            turn_order: turn_order + 1,
            phase: "ingame".to_string(),
            created_at: chrono::Utc::now(),
        }).await?;

        game_state.perform_ingame_action(&selected_action);
        turn_order += 1;
    }

    let players_in_finishing_order = get_players_in_finishing_order(game_state);

    let results: Vec<database::models::GameResultRecord> = players_in_finishing_order.iter().enumerate().map(|(place, player)| {
        let finishing_place = place + 1;
        let finishing_role = calculate_role(finishing_place, game_state.table.len());

        database::models::GameResultRecord {
            id: None,
            game_id: handle.as_i64(),
            player_id: player.state.id,
            finishing_place,
            finishing_role,
        }
    }).collect();

    recorder.finish_game(handle, &results).await?;

    Ok(())
}

fn action_type_to_string(action: &Action) -> String {
    match action {
        Action::SendCard { .. } => "SendCard".to_string(),
        Action::PlayCards { .. } => "PlayCards".to_string(),
        Action::Pass => "Pass".to_string(),
    }
}

fn serialize_card_play(card_play: &types::CardPlay) -> serde_json::Value {
    match card_play {
        types::CardPlay::Single(card) => serde_json::json!({
            "type": "Single",
            "cards": [card.to_string()]
        }),
        types::CardPlay::Pair(c1, c2) => serde_json::json!({
            "type": "Pair",
            "cards": [c1.to_string(), c2.to_string()]
        }),
        types::CardPlay::Triple(c1, c2, c3) => serde_json::json!({
            "type": "Triple",
            "cards": [c1.to_string(), c2.to_string(), c3.to_string()]
        }),
        types::CardPlay::Quad(c1, c2, c3, c4) => serde_json::json!({
            "type": "Quad",
            "cards": [c1.to_string(), c2.to_string(), c3.to_string(), c4.to_string()]
        }),
    }
}

fn serialize_card_play_single(card: &types::Card) -> serde_json::Value {
    serde_json::json!({
        "type": "Single",
        "cards": [card.to_string()]
    })
}

fn calculate_role(finishing_place: usize, num_players: usize) -> String {
    match finishing_place {
        1 => "President".to_string(),
        2 => "VicePresident".to_string(),
        place if place == num_players - 1 => "ViceAsshole".to_string(),
        place if place == num_players => "Asshole".to_string(),
        _ => "Secretary".to_string(),
    }
}

fn get_players_in_finishing_order(game_state: &GameState) -> Vec<&Player> {
    let mut worst_to_first = Vec::new();

    for player in &game_state.table {
        if !player.state.current_hand.is_empty() {
            worst_to_first.push(player);
        }
    }

    for event in game_state.history.iter().rev() {
        if matches!(event.action, Action::PlayCards { .. })
            && !worst_to_first.iter().any(|p| p.state.id == event.player_id)
        {
            if let Some(player) = game_state.get_player(event.player_id) {
                worst_to_first.push(player);
            }
        }
    }

    worst_to_first.into_iter().rev().collect()
}

pub fn generate_deck_seed() -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p simulation integration_tests::test_simulation_uses_new_database_writer`
Expected: PASS

**Step 5: Commit**

```bash
git add simulation/src/lib.rs simulation/src/tests/integration_tests.rs
git commit -m "feat: update simulation to use new DatabaseWriter interface"
```

### Task 8: Remove old GameRecorder code and clean up

**Files:**
- Remove: `database/src/repository.rs`
- Modify: `database/src/lib.rs` (remove GameRecorder trait export)
- Modify: `database/src/lib.rs` (remove NoopRecorder if not needed)

**Step 1: Remove old repository module**

```bash
rm database/src/repository.rs
```

**Step 2: Update lib.rs to remove old exports**

Remove old GameRecorder trait and related exports from `database/src/lib.rs`, keeping only the new DatabaseWriter-based exports.

**Step 3: Run full test suite**

Run: `cargo test -p database -p simulation`
Expected: All tests pass

**Step 4: Commit**

```bash
git add database/src/lib.rs
git commit -m "refactor: remove old GameRecorder code"
```

### Task 9: Add comprehensive integration tests

**Files:**
- Create: `database/src/tests/integration/end_to_end_tests.rs`
- Create: `database/src/tests/integration/transaction_rollback_tests.rs`
- Create: `database/src/tests/integration/performance_comparison_tests.rs`

**Step 1: Write end-to-end integration test**

**Step 2: Write transaction rollback test**

**Step 3: Write performance comparison test**

**Step 4: Run integration tests**

**Step 5: Commit**

### Task 10: Update configuration and documentation

**Files:**
- Modify: `database/src/config.rs` (add writer selection)
- Create: `README_DATABASE.md` (explain new architecture)
- Update: `AGENTS.md` (reflect new database structure)

**Step 1: Add writer configuration**

**Step 2: Write documentation**

**Step 3: Update project docs**

**Step 4: Commit**

---

This plan provides a complete, step-by-step implementation guide that an LLM agent can execute using TDD principles with frequent commits and testing at each stage.
