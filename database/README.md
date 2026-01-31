# Database Crate

This crate provides database persistence for the card game simulation, implementing a flexible `DatabaseWriter` trait with both bulk and streaming implementations.

## Overview

The database crate has been refactored to use a new transactional approach where game events are collected and persisted atomically, replacing the previous per-transaction recording model.

## Architecture

### DatabaseWriter Trait

The core trait that all database writers implement:

```rust
#[async_trait]
pub trait DatabaseWriter: Send + Sync {
    async fn record_player(&mut self, player_id: Uuid, name: &str) -> Result<(), DatabaseError>;
    async fn get_player_by_name(&mut self, name: &str) -> Result<Option<Uuid>, DatabaseError>;
    async fn start_game(&mut self, game_meta: GameMetadata) -> Result<GameHandle, DatabaseError>;
    async fn record_action(&mut self, handle: GameHandle, action: &ActionRecord) -> Result<(), DatabaseError>;
    async fn finish_game(&mut self, handle: GameHandle, results: &[GameResultRecord]) -> Result<(), DatabaseError>;
}
```

### Implementations

#### BulkGameWriter

Collects all game events in memory during gameplay and persists them in a single database transaction when the game finishes.

**Use case:** Ideal for simulations where you want to record complete games with minimal database overhead.

**Features:**
- In-memory event collection during gameplay
- Atomic single-transaction persistence at game end
- Automatic retry logic with exponential backoff
- Handles players, actions, results, and game metadata

#### StreamingGameWriter

Persists events to the database immediately as they occur.

**Use case:** Ideal for web applications or scenarios requiring real-time persistence and immediate data availability.

**Features:**
- Immediate database writes for each action
- Suitable for concurrent game recording
- Same retry logic for resilience

### GameHandle

A unique identifier for games that can be used to track and reference specific game sessions:

```rust
let handle = writer.start_game(metadata).await?;
writer.record_action(handle, &action).await?;
writer.finish_game(handle, &results).await?;
```

### GameEventCollector

Used internally by BulkGameWriter to collect events in memory:

```rust
pub struct GameEventCollector {
    pub metadata: GameMetadata,
    pub actions: Vec<ActionRecord>,
    pub results: Vec<GameResultRecord>,
    pub players: Vec<(Uuid, String)>,
}
```

### GameMetadata

Configuration and metadata for a game session:

```rust
pub struct GameMetadata {
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub num_players: usize,
    pub deck_seed: String,
    pub player_order: Vec<Uuid>,
    pub configuration: Option<serde_json::Value>,
}
```

## Configuration

### Writer Type Selection

You can configure which writer to use via the `DatabaseConfig`:

```rust
use database::{DatabaseConfig, DatabaseWriterType};

let config = DatabaseConfig {
    url: "sqlite://games.db".to_string(),
    pool_size: 20,
    writer_type: DatabaseWriterType::Bulk, // or Streaming
};
```

### Environment Variables

- `DATABASE_URL`: Database connection string
- `DATABASE_WRITER_TYPE`: Writer type ("bulk" or "streaming")

## Usage

### Using BulkGameWriter

```rust
use database::{BulkGameWriter, DatabaseWriter, GameMetadata};
use sqlx::SqlitePool;

let pool = SqlitePool::connect("sqlite://games.db").await?;
let mut writer = BulkGameWriter::new(pool);

// Start a game
let metadata = GameMetadata {
    started_at: chrono::Utc::now(),
    num_players: 4,
    deck_seed: "seed123".to_string(),
    player_order: vec![player1_id, player2_id, player3_id, player4_id],
    configuration: None,
};

let handle = writer.start_game(metadata).await?;

// Record actions during gameplay
writer.record_action(handle, &action).await?;

// Finish game with results
writer.finish_game(handle, &results).await?;
```

### Using StreamingGameWriter

```rust
use database::{StreamingGameWriter, DatabaseWriter, GameMetadata};
use sqlx::SqlitePool;

let pool = SqlitePool::connect("sqlite://games.db").await?;
let mut writer = StreamingGameWriter::new(pool);

// Same API as BulkGameWriter
let handle = writer.start_game(metadata).await?;
writer.record_action(handle, &action).await?;
writer.finish_game(handle, &results).await?;
```

### NoopRecorder

For testing or scenarios where persistence is not needed:

```rust
use database::NoopRecorder;

let mut writer = NoopRecorder;
// Implements DatabaseWriter but does nothing
```

## Database Schema

The database expects the following tables:

- `players`: Player information (id, name)
- `games`: Game sessions (id, started_at, finished_at, num_players, deck_seed, player_order, configuration)
- `actions`: Game actions (id, game_id, player_id, action_type, card_play, target_player_id, turn_order, phase, created_at)
- `game_results`: Game finishing results (id, game_id, player_id, finishing_place, finishing_role)

Migrations are handled via SQLx migrate in the migrations directory.

## Testing

The crate includes comprehensive tests:

```bash
# Run unit tests
cargo test -p database

# Run integration tests
cargo test -p database --test integration_tests
```

## Migration from Old API

The old `GameRecorder` trait and `DatabaseRecorder` have been removed. To migrate:

1. Replace `DatabaseRecorder` with `BulkGameWriter` or `StreamingGameWriter`
2. Update to use `&mut dyn DatabaseWriter` instead of `&dyn GameRecorder`
3. Use the `start_game` → `record_action` → `finish_game` flow
4. Handle game results as a batch at the end

See the simulation crate for a complete migration example.
