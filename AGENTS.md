# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-17
**Commit:** 07db785
**Branch:** bug/cardplay-order

## OVERVIEW
Rust workspace implementing a card game simulation (President/Asshole variant) with pluggable AI strategies and CLI interface.

## STRUCTURE
```
roosevelt/
├── Cargo.toml           # Workspace with 4 crates
├── Cargo.lock
├── rust-toolchain.toml  # Stable + rust-analyzer
├── rustfmt.toml         # Import grouping rules
├── database/            # Database persistence layer
│   ├── src/
│   │   ├── collectors/   # GameEventCollector, GameMetadata
│   │   ├── writers/      # BulkGameWriter, StreamingGameWriter
│   │   ├── config.rs     # DatabaseConfig with writer type selection
│   │   ├── error.rs      # DatabaseError types
│   │   ├── models.rs     # ActionRecord, GameResultRecord
│   │   └── lib.rs
│   └── Cargo.toml
├── types/               # Core data structures
│   ├── src/
│   │   ├── action.rs
│   │   ├── card.rs              # Custom ordering (2 highest)
│   │   ├── card_play.rs         # Single/Pair/Triple/Quad
│   │   ├── game_state.rs        # Game engine (~468 lines)
│   │   ├── hand.rs
│   │   ├── player.rs            # Strategy trait
│   │   ├── player_state.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── strategies/          # AI/human strategies
│   ├── src/
│   │   ├── lib.rs              # DefaultStrategy, RandomStrategy
│   │   └── input_strategy.rs    # Interactive CLI
│   └── Cargo.toml
└── simulation/          # CLI binary
    ├── src/
    │   ├── bin/
    │   │   └── run_simulation.rs # Entry point
    │   └── lib.rs                # run_game()
    └── Cargo.toml
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Game rules/logic | `types/src/game_state.rs` | Core engine, pregame, card passing |
| Add new strategy | `strategies/src/lib.rs` | Implement `Strategy` trait |
| CLI entry point | `simulation/src/bin/run_simulation.rs` | YAML config loading, infinite loop |
| Card ordering | `types/src/card.rs` | Two is highest rank |
| Player data | `types/src/player.rs`, `player_state.rs` | Roles, hands, public/private state |
| Database writers | `database/src/writers/` | BulkGameWriter, StreamingGameWriter |
| Game recording | `database/src/collectors/` | GameEventCollector, GameMetadata |
| Database config | `database/src/config.rs` | Writer type selection |

## CONVENTIONS
- **Workspace**: Centralized dependency management in root `Cargo.toml`
- **Strategy pattern**: Dynamic dispatch via `Box<dyn Strategy>`
- **Logging**: Use `log::` facade, initialize with `env_logger::init()`
- **Formatting**: `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`
- **Testing**: Unit and integration tests exist for database, simulation, and types crates
- **CI/CD**: `.github/workflows/ci.yml` enforces fmt, clippy, build, test

## ANTI-PATTERNS (THIS PROJECT)
- **panic!() calls**: `card_play.rs:50`, `game_state.rs:139` — prefer `Result`
- **expect() overuse**: 29 occurrences — use proper error handling
- **Mixed output**: `println!` mixed with `log::` in `input_strategy.rs`
- **TODO in error type**: `run_simulation.rs:36` — placeholder `type Err = String`
- **Role assumption**: Code assumes exactly 5 players (President/VP/Secretary/ViceAsshole/Asshole)

## UNIQUE STYLES
- **Card ordering**: Two is highest rank, then standard descending
- **Starting card**: 3♣ → 3♠ → 3♥ → 3♦ → 4♣ priority
- **Card passing**: Asshole→President (2 cards), ViceAsshole→VP (1 card)
- **YAML config**: Players defined in external `config.yaml` file

## COMMANDS
```bash
# Build all crates
cargo build

# Run simulation
cargo run --bin run_simulation -- --config config.yaml [--delay-ms 100]

# Format code
cargo fmt

# Check without building
cargo check

# Run clippy (not configured)
cargo clippy --workspace
```

## NOTES
- No `README.md`, `LICENSE`, or documentation
- No example `config.yaml` provided
- `rust-toolchain.toml` has uncommitted changes
- Branch: `bug/cardplay-order` (not `main`)
