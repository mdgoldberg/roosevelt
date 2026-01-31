use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use serde::{Deserialize, Serialize};

use database::{BulkGameWriter, DatabaseConfig, NoopRecorder};
use simulation::run_game;
use strategies::{DefaultStrategy, InputStrategy, RandomStrategy};
use types::game_state::GameState;
use types::Strategy;
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Params {
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,

    #[arg(short, long)]
    database: Option<String>,

    #[arg(short, long)]
    force_new_players: bool,

    #[arg(short, long)]
    auto_reuse_players: bool,

    #[arg(short, long)]
    delay_ms: Option<u64>,
}

#[derive(Deserialize)]
struct Config {
    game_config: serde_yaml::Value,
    database: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct GameConfig {
    players: Vec<PlayerConfig>,
    #[serde(default)]
    delay_ms: Option<u64>,
}

#[derive(Deserialize, Serialize)]
struct PlayerConfig {
    name: String,
    strategy: String,
}

#[derive(Debug)]
enum Strategies {
    Default(DefaultStrategy),
    Random(RandomStrategy),
    Input(InputStrategy),
}

impl FromStr for Strategies {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(Strategies::Default(DefaultStrategy::default())),
            "random" => Ok(Strategies::Random(RandomStrategy::default())),
            "input" => Ok(Strategies::Input(InputStrategy::default())),
            _ => Err(format!("Unable to parse {} to Strategy impl", s)),
        }
    }
}

impl From<Strategies> for Box<dyn Strategy> {
    fn from(value: Strategies) -> Self {
        match value {
            Strategies::Default(strat) => Box::new(strat) as Box<dyn Strategy>,
            Strategies::Random(strat) => Box::new(strat) as Box<dyn Strategy>,
            Strategies::Input(strat) => Box::new(strat) as Box<dyn Strategy>,
        }
    }
}

fn get_config(path: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Params::parse();
    log::info!("args: {:?}", args);
    let config = get_config(args.config.clone())?;

    let game_config: GameConfig = serde_yaml::from_value(config.game_config)?;

    let db_config = DatabaseConfig::from_cli_or_env_or_yaml(args.database, config.database);

    let mut recorder: Box<dyn database::DatabaseWriter> = if db_config.url == "sqlite::memory:" {
        log::info!("Using in-memory database (no persistence)");
        Box::new(NoopRecorder)
    } else {
        log::info!("Using database: {}", db_config.url);
        let pool = db_config.create_pool().await?;
        let recorder = BulkGameWriter::new(pool);
        recorder.run_migrations().await?;
        Box::new(recorder)
    };

    let player_inputs = register_or_reuse_players(
        &game_config.players,
        &mut *recorder,
        args.force_new_players,
        args.auto_reuse_players,
    )
    .await?;

    let mut game_state = GameState::new(player_inputs);

    loop {
        let game_config_json = serde_json::to_value(&game_config)?;

        run_game(
            &mut game_state,
            game_config.delay_ms,
            &mut *recorder,
            Some(game_config_json),
        )
        .await?;
        game_state.start_new_game();
    }
}

async fn register_or_reuse_players(
    player_configs: &[PlayerConfig],
    recorder: &mut dyn database::DatabaseWriter,
    force_new: bool,
    auto_reuse: bool,
) -> Result<Vec<(Uuid, String, Box<dyn Strategy>)>, Box<dyn std::error::Error>> {
    let mut player_inputs = Vec::new();

    for config in player_configs {
        let existing_player_id = if force_new {
            None
        } else {
            recorder.get_player_by_name(&config.name).await?
        };

        let player_id = match (existing_player_id, auto_reuse) {
            (Some(uuid), true) => {
                log::info!("Auto-reusing existing player: {} ({})", config.name, uuid);
                uuid
            }
            (Some(uuid), false) => {
                println!("Found existing player: {} ({})", config.name, uuid);
                print!("Reuse existing player? (y/n) > ");
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    let new_uuid = Uuid::new_v4();
                    recorder.record_player(new_uuid, &config.name).await?;
                    log::info!("Created new player: {} ({})", config.name, new_uuid);
                    new_uuid
                } else {
                    uuid
                }
            }
            (None, _) => {
                let new_uuid = Uuid::new_v4();
                recorder.record_player(new_uuid, &config.name).await?;
                log::info!("Created new player: {} ({})", config.name, new_uuid);
                new_uuid
            }
        };

        let strategy = config.strategy.parse::<Strategies>()?.into();
        player_inputs.push((player_id, config.name.clone(), strategy));
    }

    Ok(player_inputs)
}
