# TYPES CRATE

**Purpose:** Core data structures and game logic for card game simulation

## OVERVIEW
Defines all game types: Action, Card, CardPlay, GameState, Player, Strategy trait, Roles, and game engine (~1365 LOC total).

## STRUCTURE
```
types/src/
├── action.rs          # Action enum (SendCard, PlayCards, Pass)
├── card.rs            # Card wrapper with custom ordering (Two highest)
├── card_play.rs       # CardPlay (Single, Pair, Triple, Quad) + Ord impl
├── game_state.rs      # GameState, PublicInfo, Event (~468 lines)
├── hand.rs            # Hand trait, combinations generation
├── player.rs          # Player struct + Strategy trait
├── player_state.rs    # PlayerState, PublicPlayerState, Role enum
└── lib.rs             # Exports all types
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Game loop | `game_state.rs:37+` | `GameState::new()`, `run_pregame()`, `still_playing()` |
| Action logic | `game_state.rs:100+` | `perform_ingame_action()`, validation |
| Card passing | `game_state.rs:200+` | Pregame phase, role-based card exchange |
| New game setup | `game_state.rs:350+` | Role assignment, deck reset |
| Strategy trait | `player.rs` | `select_action()` signature |
| Card ordering | `card.rs` | `value()` method (Two = 52, Three = 3) |

## CONVENTIONS
- **Custom ordering**: Two is highest (value=52), then Ace(51), King(50)...Three(3)
- **Immutable actions**: Actions cloned before passing to strategies
- **Public/private separation**: `PublicInfo` excludes private player state
- **Assert invariants**: `assert_eq!` for card counts, `assert!` for game rules

## ANTI-PATTERNS (THIS CRATE)
- **panic!()**: `card_play.rs:50` (invalid CardPlay), `game_state.rs:139` (invalid send)
- **TODO**: `game_state.rs:360` (shuffle seating order)
- **NOTE**: `game_state.rs:395` (assumes all roles used)
- **Commented code**: `action.rs:46-52` (incomplete PartialOrd impl)

## UNIQUE STYLES
- **Starting card detection**: 3♣ → 3♠ → 3♥ → 3♦ → 4♣ priority order
- **Role-based passing**: Asshole sends 2 to President, ViceAsshole sends 1 to VP
- **Event logging**: `history: Vec<Event>` tracks all actions

## NOTES
- `GameState` owns players via `VecDeque<Player>`
- `Player::strategy` is `Box<dyn Strategy>` for runtime polymorphism
- Hand detection uses itertools combinations
