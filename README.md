# Roosevelt

A Rust-based card game simulation implementing the President/Asshole card game with pluggable AI strategies.

## What is Roosevelt?

Roosevelt simulates the classic "President" (also known as "Asshole") card game where players compete to empty their hands first. The game features:

- **Multiple AI strategies** (Random, Default, and Interactive)
- **Role-based card passing** between games (President/VP/Secretary/Vice-Asshole/Asshole)
- **Configurable player setups** via YAML files
- **Real-time game visualization** with optional delays

## Quick Start

### Prerequisites

- Rust 1.92.0 or later
- Cargo (included with Rust)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd roosevelt

# Build the project
cargo build --release
```

### Running Your First Game

1. Create a configuration file `config.yaml`:

```yaml
players:
  - name: "Alice"
    strategy: "default"
  - name: "Bob"
    strategy: "random"
  - name: "Charlie"
    strategy: "input"  # Interactive player (you!)
```

2. Run the simulation:

```bash
cargo run --release --bin run_simulation -- --config config.yaml
```

3. For a slower game with delays between moves:

```bash
cargo run --release --bin run_simulation -- --config config.yaml --delay-ms 500
```

## Game Rules Overview

### Objective
Be the first player to play all cards from your hand. Players are ranked based on finishing order, which determines card passing in the next game.

### Card Ranking
- **Two is the highest card** (unlike standard poker rankings)
- Standard descending order: Two → Ace → King → ... → Three (lowest)

### Gameplay
1. **Starting Player**: The player with 3♣ (or 3♠/3♥/3♦/4♣ in that priority) starts
2. **First Play**: Must contain the starting card
3. **Card Plays**: You can play:
   - Single cards
   - Pairs (two matching ranks)
   - Triples (three matching ranks)
   - Quads (four matching ranks)
4. **Beating Cards**: Played cards must be **higher rank** than the current top cards
5. **Passing**: You can pass on your turn
6. **New Round**: When everyone passes, the last player starts a new round

### Roles and Card Passing

After each game, players receive roles based on finishing order:

| Role | Cards to Send | Cards to Receive |
|------|---------------|-------------------|
| President | 2 cards (best) | From Asshole |
| Vice President | 1 card (best) | From Vice-Asshole |
| Secretary | - | - |
| Vice-Asshole | 1 card (worst) | To Vice President |
| Asshole | 2 cards (worst) | To President |

## Available Strategies

### Default Strategy
Conservative AI that:
- Always plays the **lowest** allowable card(s)
- Sends the **lowest** card during pregame passing
- Good for learning the game basics

### Random Strategy
Chaotic AI that:
- Randomly selects from all valid actions
- Useful for testing game mechanics
- Unpredictable gameplay

### Input Strategy
**Interactive human player** that:
- Prompts you via the command line
- Enter actions as:
  - `play <cards>` - e.g., `play 3♠ 4♥` or `play K♠ K♥`
  - `send <card>` - e.g., `send 2♦` (during card passing phase)
  - `pass` - Pass your turn

**Card input format**: Use standard suit symbols (♠ ♥ ♦ ♣) or abbreviations (S H D C)
- Examples: `3♠`, `K♥`, `2♦`, `10♣`

## Configuration

The `config.yaml` file defines players and their strategies:

```yaml
players:
  - name: "Player 1"
    strategy: "default"   # Conservative AI
  - name: "Player 2"
    strategy: "random"     # Chaotic AI
  - name: "Player 3"
    strategy: "input"      # Interactive human
```

### Strategy Options
- `default` - Conservative AI (plays worst allowable cards)
- `random` - Random action selection
- `input` - Interactive human player (requires terminal input)

## Command Line Options

```bash
cargo run --bin run_simulation -- [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `--config <path>` | Path to YAML config file | `config.yaml` |
| `--delay-ms <ms>` | Delay between moves (milliseconds) | No delay |

## Example Games

### 3-Player Game (Quick)
```yaml
players:
  - name: "You"
    strategy: "input"
  - name: "Bot 1"
    strategy: "default"
  - name: "Bot 2"
    strategy: "random"
```

### 5-Player Game (Full Roles)
```yaml
players:
  - name: "President"
    strategy: "default"
  - name: "VP"
    strategy: "default"
  - name: "Secretary"
    strategy: "random"
  - name: "Vice-Asshole"
    strategy: "random"
  - name: "Asshole"
    strategy: "input"
```

### All Random Game (Fast Simulation)
```yaml
players:
  - name: "Random 1"
    strategy: "random"
  - name: "Random 2"
    strategy: "random"
  - name: "Random 3"
    strategy: "random"
  - name: "Random 4"
    strategy: "random"
```

## Tips for Playing

1. **Start with low cards**: Don't waste your high cards early
2. **Save Twos**: Twos are the highest cards, save them to beat powerful plays
3. **Plan your exit**: Try to play out cards so you can go out first
4. **Watch the passing phase**: As Asshole, you must give away your best cards!
5. **Use `--delay-ms`**: If the game moves too fast to follow, add a delay

## Current Limitations

- Assumes exactly 5 players for full role system
- No shuffle option for seating order between games
- No tournament or league mode (runs infinitely until interrupted)
- Card passing assumes all roles are present

## Development

For developers interested in extending Roosevelt:

- **Adding strategies**: Implement the `Strategy` trait in `strategies/src/lib.rs`
- **Modifying game rules**: Edit `types/src/game_state.rs`
- **CLI changes**: Update `simulation/src/bin/run_simulation.rs`

The project uses a Rust workspace with three crates:
- `types` - Core game logic and data structures
- `strategies` - AI player implementations
- `simulation` - CLI game runner

## License

See LICENSE file for details.

## Contributing

Contributions welcome! Areas for improvement:
- Additional AI strategies
- Tournament mode
- Network multiplayer
- GUI interface
- Statistics and replay analysis
