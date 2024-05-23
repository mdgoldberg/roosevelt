use std::{thread::sleep, time::Duration};

use types::GameState;

pub fn run_game(game_state: &mut GameState) {
    assert_eq!(game_state.history.len(), 0);
    game_state.run_pregame();
    while game_state.still_playing() {
        log::debug!("{game_state}");
        sleep(Duration::from_millis(100));
        let available_actions = game_state.permitted_actions();
        let public_info = game_state.public_info();
        let current_player = game_state.current_player_mut();
        let selected_action = current_player.strategy.select_action(
            &current_player.state,
            &public_info,
            &available_actions,
        );
        game_state.perform_ingame_action(&selected_action);
    }
    game_state.start_new_game();
}
