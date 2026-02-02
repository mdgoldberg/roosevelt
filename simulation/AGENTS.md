# SIMULATION CRATE

**Purpose:** CLI binary to run card game simulations

## OVERVIEW
Binary crate with YAML config loading, strategy instantiation, and infinite game loop using `run_game()` from library.

## STRUCTURE
```
simulation/src/
├── bin/
│   └── run_simulation.rs    # fn main(), CLI args, YAML config
└── lib.rs                    # run_game() function (~25 lines)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| CLI args | `run_simulation.rs:9-15` | `--config`, `--delay-ms` |
| Strategy parsing | `run_simulation.rs:29-56` | FromStr for Strategies enum |
| Config loading | `run_simulation.rs:58-61` | YAML deserialization |
| Main loop | `run_simulation.rs:63-85` | Infinite `run_game()` calls |
| Game runner | `lib.rs:5-24` | Prephase → game loop → new game |

## CONVENTIONS
- **Config-driven**: Players defined in external YAML (default: `config.yaml`)
- **Strategy polymorphism**: `Box<dyn Strategy>` via `From<Strategies>` impl
- **Logging**: `env_logger::init()` at startup
- **Optional delay**: `--delay-ms` argument for slowed gameplay

## ANTI-PATTERNS (THIS CRATE)
- **TODO in error type**: `run_simulation.rs:36` — placeholder `type Err = String`
- **expect() overuse**: File open, YAML parse, strategy parse all `expect()`
- **No config.example**: Users must guess YAML format
- **Mixed output**: No `println!` but log facade used inconsistently

## UNIQUE STYLES
- **Strategy enum**: `Strategies` (Default/Random/Input) with `FromStr` impl
- **Runtime dispatch**: `Box<dyn Strategy>` allows mixing strategies per game
- **Infinite loop**: Never exits, runs `run_game()` forever until Ctrl+C

## COMMANDS
```bash
# Run with config
cargo run --bin run_simulation -- --config config.yaml

# With delay between moves
cargo run --bin run_simulation -- --config config.yaml --delay-ms 100
```

## CONFIG FORMAT
Expected YAML structure (no example file exists):
```yaml
players:
  - name: "Alice"
    strategy: "default"    # or "random" or "input"
  - name: "Bob"
    strategy: "random"
```

## NOTES
- Entry point: `fn main()` at line 63
- Uses `clap` for CLI argument parsing
- `lib.rs` is minimal wrapper calling `GameState` methods
