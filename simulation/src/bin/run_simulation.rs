use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use serde::Deserialize;
use simulation::run_game;
use strategies::{DefaultStrategy, InputStrategy, RandomStrategy};
use types::{GameState, Strategy};

#[derive(Parser, Debug)]
struct Params {
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

#[derive(Deserialize)]
struct PlayerConfig {
    name: String,
    strategy: String,
}

#[derive(Deserialize)]
struct PlayersConfig {
    players: Vec<PlayerConfig>,
}

#[derive(Debug)]
pub enum Strategies {
    Default(DefaultStrategy),
    Random(RandomStrategy),
    Input(InputStrategy),
}

impl FromStr for Strategies {
    type Err = String; // TODO

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(Strategies::Default(DefaultStrategy::default())),
            "random" => Ok(Strategies::Random(RandomStrategy::default())),
            "input" => Ok(Strategies::Input(InputStrategy::default())),
            _ => Err(format!("Unable to parse {s:?} to Strategy impl")),
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

fn get_config(path: PathBuf) -> PlayersConfig {
    let f = std::fs::File::open(path).expect("File to open");
    serde_yaml::from_reader(f).expect("File to parse to PlayersConfig")
}

fn main() {
    env_logger::init();
    let args = Params::parse();
    log::info!("args: {args:?}");
    let config = get_config(args.config);
    let player_inputs: Vec<(String, Box<dyn Strategy>)> = config
        .players
        .into_iter()
        .map(|player_conf| {
            (
                player_conf.name,
                (&player_conf.strategy)
                    .parse::<Strategies>()
                    .expect("Unable to parse strategy")
                    .into(),
            )
        })
        .collect();
    let mut game_state = GameState::new(player_inputs);
    loop {
        run_game(&mut game_state);
    }
}
