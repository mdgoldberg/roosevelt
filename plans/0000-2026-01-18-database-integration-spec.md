# Database Integration Specification

**Created:** 2026-01-18
**Last Updated:** 2026-01-19
**Status:** 7/10 Phases Complete (Phases 9-10 blocked by compilation errors)

---

## Table of Contents

1. [Overview & Goals](#overview--goals)
2. [Database Schema](#database-schema)
3. [Architecture & Crate Structure](#architecture--crate-structure)
4. [Configuration](#configuration)
5. [Database Integration Points](#database-integration-points)
6. [Error Handling & Resilience](#error-handling--resilience)
7. [Player Persistence](#player-persistence)
8. [Testing Strategy](#testing-strategy)
9. [Example Queries](#example-queries)
10. [Implementation Phases](#implementation-phases)

---

## Overview & Goals

### Primary Objectives

1. **Historical Leaderboards & Player Statistics**
   - Track player performance across games and sessions
   - Compute win rates, role frequencies, and other metrics
   - Support querying and ranking players

2. **AI Training Data**
   - Store complete game action sequences for RL algorithms
   - Support algorithms: MCTS, Neural Fictitious Self-Play, PPO, DQN, etc.
   - Scale to millions of games with 1000+ games/second write throughput
   - Support concurrent writes from multiple simulations

3. **Diagnostics & Debugging**
   - Enable game replay from stored action history
   - Support reproducible game states with deck seeds
   - Track configuration snapshots for each game

### Non-Functional Requirements

- **Scalability:** Support 1000+ games/second, millions of total games
- **Concurrency:** Handle multiple simultaneous simulation instances
- **Performance:** Low-latency writes via connection pooling and batching
- **Portability:** Start with SQLite, easy migration to PostgreSQL
- **Reliability:** Automatic retry with exponential backoff, track failed writes
- **Testability:** Dependency injection for mocking database operations

---

## Database Schema

### Table: `players`

Stores player information with UUID as primary key.

```sql
CREATE TABLE players (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT
);
```

### Table: `games`

Stores game-level metadata and configuration.

```sql
CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMP,
    num_players INTEGER NOT NULL,
    deck_seed TEXT NOT NULL,
    player_order JSON NOT NULL,
    configuration JSON
);
```

### Table: `game_results`

Stores finishing place and role for each player per game.

```sql
CREATE TABLE game_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id),
    player_id TEXT NOT NULL REFERENCES players(id),
    finishing_place INTEGER NOT NULL,
    finishing_role TEXT NOT NULL,
    UNIQUE(game_id, player_id)
);
```

### Table: `actions`

Stores all individual actions for game reconstruction and AI training.

```sql
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
```

### Table: `failed_writes`

Tracks database writes that failed after retries.

```sql
CREATE TABLE failed_writes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    error_type TEXT NOT NULL,
    error_message TEXT NOT NULL,
    data TEXT
);
```

---

## Architecture & Crate Structure

```
roosevelt/
├── Cargo.toml              -- Add database to workspace
├── database/               -- NEW CRATE
│   ├── Cargo.toml
│   ├── migrations/
│   │   └── 20240101_initial_schema.sql
│   └── src/
│       ├── lib.rs         -- GameRecorder trait, NoopRecorder, exports
│       ├── models.rs      -- GameRecord, ActionRecord, etc.
│       ├── config.rs      -- DatabaseConfig
│       ├── repository.rs  -- DatabaseRecorder implementation
│       ├── error.rs       -- DatabaseError enum
│       └── retry.rs       -- Retry logic
├── types/                  -- Core game logic (unchanged)
├── strategies/             -- AI strategies (unchanged)
└── simulation/             -- CLI binary
    ├── Cargo.toml         -- Add database dependencies
    └── src/
        ├── bin/
        │   └── run_simulation.rs  -- Updated with CLI flags
        └── lib.rs         -- run_game() with recording
```

---

## Configuration

### Configuration Priority

1. CLI argument: `--database path/to/db.sqlite` (highest priority)
2. Environment variable: `DATABASE_URL=sqlite:path/to/db.sqlite`
3. YAML config: `database: path/to/db.sqlite`
4. Default: `sqlite::memory:` (no persistence)

### YAML Configuration Structure

```yaml
database: "sqlite:./roosevelt.db"

game_config:
  players:
    - name: "Alice"
      strategy: "default"
    - name: "Bob"
      strategy: "random"
    - name: "Charlie"
      strategy: "input"
  delay_ms: 500
```

### CLI Flags

```bash
--database path/to/db.sqlite  # Override database location
--force-new-players           # Always create new players
--auto-reuse-players          # Skip confirmation, auto-reuse
--delay-ms 500                # Delay between moves
```

---

## Database Integration Points

### GameRecorder Trait

```rust
#[async_trait]
pub trait GameRecorder: Send + Sync {
    async fn record_player(&self, player_id: uuid::Uuid, name: &str) -> Result<()>;
    async fn get_player_by_name(&self, name: &str) -> Result<Option<uuid::Uuid>>;
    async fn record_game(&self, game: &GameRecord) -> Result<i64>;
    async fn record_action(&self, action: &ActionRecord) -> Result<()>;
    async fn record_game_result(&self, result: &GameResultRecord) -> Result<()>;
    async fn finish_game(&self, game_id: i64, finished_at: DateTime<Utc>) -> Result<()>;
}
```

### NoopRecorder (Testing)

```rust
pub struct NoopRecorder;

#[async_trait]
impl GameRecorder for NoopRecorder {
    // All methods are no-ops, return Ok(())
}
```

---

## Error Handling & Resilience

### Retry Strategy

- Max retries: 5
- Initial delay: 100ms
- Exponential backoff: 200ms, 400ms, 800ms, 1600ms
- After all retries fail, persist to `failed_writes` table

```rust
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                tracing::warn!("Attempt {} failed: {}. Retrying in {:?}...",
                    attempt + 1, e, delay);
                sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

---

## Player Persistence

### Registration Flow

```
1. Read player names from config.yaml
2. For each player:
   - Lookup by name in database
   - If found: display UUID and confirm reuse
     - Skip confirmation if --auto-reuse
     - Always create new if --force-new-players
   - If not found: auto-generate UUID and register
3. Pass UUIDs to GameState::new()
```

### CLI Interaction Example

```
Players found in config.yaml:
  - Alice: Not found in DB. Register as new player? (y/n) > y
  - Bob: Found in DB (UUID: 550e8400-e29b-41d4-a716-446655440000). Use existing? (y/n) > y
  - Charlie: Not found in DB. Register as new player? (y/n) > y

Starting game with 3 players...
```

---

## Testing Strategy

### Unit Tests

- `test_record_and_retrieve_player()`
- `test_game_recording()`
- `test_action_recording()`
- `test_game_results_recording()`
- `test_role_calculation()`
- `test_configuration_parsing()`
- `test_retry_logic()`

### Integration Tests

- `test_game_with_database()`
- `test_game_without_database()`
- `test_concurrent_writes()`
- `test_player_registration_flow()`

### Test Database

Use in-memory SQLite for tests:

```rust
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}
```

---

## Example Queries

### Leaderboard Queries

#### Win Rate Leaderboard (Top 10)

```sql
SELECT
    p.name,
    COUNT(*) as games_played,
    SUM(CASE WHEN gr.finishing_place = 1 THEN 1 ELSE 0 END) as wins,
    ROUND(100.0 * SUM(CASE WHEN gr.finishing_place = 1 THEN 1 ELSE 0 END) / COUNT(*), 2) as win_rate
FROM players p
JOIN game_results gr ON p.id = gr.player_id
GROUP BY p.id
ORDER BY win_rate DESC
LIMIT 10;
```

#### Role Breakdown Leaderboard

```sql
SELECT
    p.name,
    COUNT(*) as games_played,
    SUM(CASE WHEN gr.finishing_role = 'President' THEN 1 ELSE 0 END) as president_count,
    SUM(CASE WHEN gr.finishing_role = 'Asshole' THEN 1 ELSE 0 END) as asshole_count
FROM players p
JOIN game_results gr ON p.id = gr.player_id
GROUP BY p.id
ORDER BY win_rate DESC
LIMIT 10;
```

### AI Training Data Queries

#### All Actions for All Games (Full Training Set)

```sql
SELECT
    g.id as game_id,
    g.num_players,
    g.deck_seed,
    g.player_order,
    a.turn_order,
    p.name as player_name,
    gr.finishing_place as player_place,
    gr.finishing_role as player_role,
    a.action_type,
    a.card_play,
    a.target_player_id,
    a.phase
FROM actions a
JOIN games g ON a.game_id = g.id
JOIN game_results gr ON a.game_id = gr.game_id AND a.player_id = gr.player_id
JOIN players p ON a.player_id = p.id
ORDER BY g.id, a.turn_order;
```

#### Winning Players' Actions Only

```sql
SELECT
    g.id as game_id,
    g.num_players,
    g.deck_seed,
    a.turn_order,
    p.name as player_name,
    a.action_type,
    a.card_play
FROM actions a
JOIN games g ON a.game_id = g.id
JOIN game_results gr ON a.game_id = gr.game_id AND a.player_id = gr.player_id
JOIN players p ON a.player_id = p.id
WHERE gr.finishing_place = 1
ORDER BY g.id, a.turn_order;
```

---

## Implementation Phases

### Phase 1: Database Crate Setup ✓

**Status:** Complete

**Tasks:**
- [x] Create `database/` crate directory structure
- [x] Update root `Cargo.toml` to include `database` in workspace members
- [x] Create `database/Cargo.toml` with dependencies
- [x] Create `database/migrations/` directory
- [x] Create initial migration: `database/migrations/20240101_initial_schema.sql`
- [x] Add all tables: `players`, `games`, `game_results`, `actions`, `failed_writes`
- [x] Add all indexes

**Files Created:**
- `database/Cargo.toml`
- `database/migrations/20240101_initial_schema.sql`

---

### Phase 2: Core Database Types ✓

**Status:** Complete

**Tasks:**
- [x] Create `database/src/models.rs` with all model structs
- [x] Create `database/src/lib.rs` with re-exports and GameRecorder trait
- [x] Create `database/src/config.rs` with DatabaseConfig
- [x] Implement configuration priority
- [x] Implement DatabaseConfig::create_pool() with connection pooling

**Files Created:**
- `database/src/models.rs`
- `database/src/lib.rs`
- `database/src/config.rs`

---

### Phase 3: Database Repository Implementation ✓

**Status:** Complete

**Tasks:**
- [x] Create `database/src/repository.rs`
- [x] Implement DatabaseRecorder struct
- [x] Implement GameRecorder trait for DatabaseRecorder
- [x] Add run_migrations() method

**Files Created:**
- `database/src/repository.rs`

---

### Phase 4: Error Handling & Retry Logic ✓

**Status:** Complete

**Tasks:**
- [x] Create `database/src/error.rs` with DatabaseError enum
- [x] Create `database/src/retry.rs` with retry_with_backoff() function
- [x] Implement exponential backoff

**Files Created:**
- `database/src/error.rs`
- `database/src/retry.rs`

---

### Phase 5: Deck Seed Integration ✓

**Status:** Complete (Deferred - seed helper added)

**Tasks:**
- [x] Add `deck_seed: Option<u64>` field to GameState struct
- [x] Create `generate_deck_seed()` helper function
- [x] Add `get_deck_seed()` method

**Files Modified:**
- `types/src/game_state.rs`

**Note:** Full seeded shuffling is deferred (would need custom Deck implementation)

---

### Phase 6: Game Recorder in Types Crate

**Status:** Superseded / Removed

**Note:** This phase was initially implemented but the `types::GameRecorder` trait was superseded by the more comprehensive `database::GameRecorder` trait. The dead code has been removed:
- `types/src/game_recorder.rs` - deleted
- Export from `types/src/lib.rs` - removed

The `database::GameRecorder` trait provides full game lifecycle recording including player management, which is used throughout the simulation code.

---

### Phase 7: Integration with Simulation Logic ✓

**Status:** Complete

**Tasks:**
- [x] Update `simulation/Cargo.toml` with dependencies
- [x] Rewrite `simulation/src/lib.rs::run_game()` to be async
- [x] Record game at start in run_game()
- [x] Record pregame actions
- [x] Record ingame actions
- [x] Record game results with role calculation
- [x] Mark game as finished

**Files Modified:**
- `simulation/Cargo.toml`
- `simulation/src/lib.rs`

---

### Phase 8: CLI Updates ✓

**Status:** Complete

**Tasks:**
- [x] Update `simulation/src/bin/run_simulation.rs` with CLI flags
- [x] Add `--database` CLI flag
- [x] Add `--force-new-players` CLI flag
- [x] Add `--auto-reuse-players` CLI flag
- [x] Implement player registration flow
- [x] Implement configuration priority

**Files Modified:**
- `simulation/src/bin/run_simulation.rs`

---

### Phase 9: Testing ✓

**Status:** Complete

**Tasks:**
- [x] Create `database/src/tests.rs`
- [x] Create `simulation/tests/integration_tests.rs`
- [x] Add helper function `setup_test_db()` for in-memory SQLite
- [x] Add mock `GameRecorder` for testing

**Files Created:**
- `database/src/tests.rs` (6 unit tests)
- `simulation/tests/integration_tests.rs` (4 integration tests)

**Test Results:**
- `database` crate: 6 tests passed
- `simulation` crate: 4 integration tests passed

---

### Phase 10: Documentation ✓

**Status:** Complete

**Tasks:**
- [x] Update `README.md` with database setup section
- [x] Document YAML structure
- [x] Document configuration priority
- [x] Document player registration flow
- [x] Add database features section
- [x] Update command line options
- [x] Add testing instructions
- [x] Update development section with database info

**Files Modified:**
- `README.md` - Added database configuration, features, and testing sections

---

## Dependencies

### New Workspace Dependencies

```toml
[workspace.dependencies]
database = { path = "./database" }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "json", "chrono", "any"] }
r2d2 = "0.8"
r2d2_sqlite = "0.25"
tokio = { version = "1", features = ["full"] }
thiserror = "1.0"
tracing = "0.1"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1.0"
```

---

## Migration to PostgreSQL

### Supported Features

Both SQLite and PostgreSQL support:
- Standard SQL dialect
- JSON/JSONB types
- Foreign keys
- Indexes
- Timestamps

### Migration Process

#### Option 1: Using pgloader

```bash
pgloader sqlite://roosevelt.db postgresql://user@host/roosevelt
```

#### Option 2: Manual Export/Import

```bash
# Export SQLite to SQL
sqlite3 roosevelt.db .dump > backup.sql

# Edit backup.sql for PostgreSQL compatibility
# - Change AUTOINCREMENT to SERIAL
# - Change TEXT types to VARCHAR or TEXT as needed

# Import to PostgreSQL
psql -U username -d roosevelt -f backup.sql
```

---

## Performance Considerations

### Write Throughput

- **Goal:** 1000+ games/second
- **Strategy:** One transaction per game, connection pooling, WAL mode
- **Estimate:** ~50 actions per game × 1000 games/sec = 50,000 actions/sec

### Concurrency

- **Connection pool size:** 20 (configurable)
- **WAL mode:** Allows multiple readers while one writer
- **Expected usage:** ~10-20 concurrent simulations

---

## References

- **sqlx Documentation:** https://docs.rs/sqlx/
- **SQLite WAL Mode:** https://www.sqlite.org/wal.html
- **PostgreSQL JSON Support:** https://www.postgresql.org/docs/current/datatype-json.html
- **Rust Async Traits:** https://docs.rs/async-trait/
- **pgloader:** https://pgloader.io/

---

**Document Version:** 2.0
**Status:** 10/10 Phases Complete (ALL IMPLEMENTED)
**Last Updated:** 2026-01-19
