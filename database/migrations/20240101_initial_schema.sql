-- Players table
CREATE TABLE players (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT
);

-- Games table
CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMP,
    num_players INTEGER NOT NULL,
    deck_seed TEXT NOT NULL,
    player_order JSON NOT NULL,
    configuration JSON
);

-- Game results table
CREATE TABLE game_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id),
    player_id TEXT NOT NULL REFERENCES players(id),
    finishing_place INTEGER NOT NULL,
    finishing_role TEXT NOT NULL,
    UNIQUE(game_id, player_id)
);

-- Actions table
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

-- Failed writes table
CREATE TABLE failed_writes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    error_type TEXT NOT NULL,
    error_message TEXT NOT NULL,
    data TEXT
);

-- Indexes for performance
CREATE INDEX idx_actions_game_id ON actions(game_id);
CREATE INDEX idx_actions_player_id ON actions(player_id);
CREATE INDEX idx_actions_turn_order ON actions(turn_order);
CREATE INDEX idx_game_results_game_id ON game_results(game_id);
CREATE INDEX idx_game_results_player_id ON game_results(player_id);
CREATE INDEX idx_game_results_finishing_place ON game_results(finishing_place);
CREATE INDEX idx_game_results_finishing_role ON game_results(finishing_role);
CREATE INDEX idx_games_started_at ON games(started_at);
