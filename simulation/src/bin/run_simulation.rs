use clap::Parser;

use simulation::types::GameState;

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
    let mut game_state = GameState::new(&player_names);
    loop {
        while game_state.still_playing() {
            let available_actions = game_state.permitted_actions();
            let selected_action = game_state
                .current_player()
                .select_action(&game_state, &available_actions);
            game_state.perform_action(&selected_action);
        }
        game_state.start_new_game();
    }
}
