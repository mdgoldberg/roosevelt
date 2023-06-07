use clap::Parser;
use env_logger;
use log;
use roosevelt::types::GameState;

#[derive(Parser, Debug)]
struct Params {
    #[arg(short, long)]
    player: Vec<String>,
}

fn main() {
    env_logger::init();
    let args = Params::parse();
    log::info!("args: {args:?}");
    let player_names = args.player.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    log::info!("players: {player_names:?}");
    let mut game_state = GameState::new(&player_names);
    log::info!("Game state: {game_state:?}");
    game_state.run_pregame();
    log::info!("Game state after pregame: {game_state:?}");
}
