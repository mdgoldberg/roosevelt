use database::NoopRecorder;
use simulation::run_game;
use strategies::{DefaultStrategy, RandomStrategy};
use types::game_state::GameState;
use uuid::Uuid;

fn make_player_inputs(
    names_and_strategies: Vec<(&str, Box<dyn types::Strategy>)>,
) -> Vec<(Uuid, String, Box<dyn types::Strategy>)> {
    names_and_strategies
        .into_iter()
        .map(|(name, strategy)| (Uuid::new_v4(), name.to_string(), strategy))
        .collect()
}

#[tokio::test]
async fn test_run_game_with_default_strategies() {
    let player_inputs = make_player_inputs(vec![
        (
            "Alice",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
        (
            "Bob",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
        (
            "Charlie",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
    ]);

    let mut game_state = GameState::new(player_inputs);
    let mut recorder = NoopRecorder;

    run_game(&mut game_state, None, &mut recorder, None)
        .await
        .expect("Game should complete successfully");

    assert!(!game_state.still_playing());
}

#[tokio::test]
async fn test_run_game_with_mixed_strategies() {
    let player_inputs = make_player_inputs(vec![
        (
            "Alice",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
        (
            "Bob",
            Box::new(RandomStrategy::default()) as Box<dyn types::Strategy>,
        ),
        (
            "Charlie",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
    ]);

    let mut game_state = GameState::new(player_inputs);
    let mut recorder = NoopRecorder;

    run_game(&mut game_state, None, &mut recorder, None)
        .await
        .expect("Game should complete successfully");

    assert!(!game_state.still_playing());
}

#[tokio::test]
async fn test_run_game_with_delay() {
    let player_inputs = make_player_inputs(vec![
        (
            "Alice",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
        (
            "Bob",
            Box::new(RandomStrategy::default()) as Box<dyn types::Strategy>,
        ),
    ]);

    let mut game_state = GameState::new(player_inputs);
    let mut recorder = NoopRecorder;

    let start = std::time::Instant::now();
    run_game(&mut game_state, Some(10), &mut recorder, None)
        .await
        .expect("Game should complete successfully");
    let duration = start.elapsed();

    assert!(!game_state.still_playing());

    let num_players = game_state.table.len();
    let estimated_min_delay_ms = num_players as u64 * 10;
    assert!(
        duration.as_millis() >= estimated_min_delay_ms as u128,
        "Game should take at least {}ms with delays",
        estimated_min_delay_ms
    );
}

#[tokio::test]
async fn test_multiple_games() {
    let player_inputs = make_player_inputs(vec![
        (
            "Alice",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
        (
            "Bob",
            Box::new(RandomStrategy::default()) as Box<dyn types::Strategy>,
        ),
    ]);

    let mut game_state = GameState::new(player_inputs);
    let mut recorder = NoopRecorder;

    run_game(&mut game_state, None, &mut recorder, None)
        .await
        .expect("First game should complete successfully");

    assert!(!game_state.still_playing());

    game_state.start_new_game();

    let player_inputs_2 = make_player_inputs(vec![
        (
            "Alice",
            Box::new(DefaultStrategy::default()) as Box<dyn types::Strategy>,
        ),
        (
            "Bob",
            Box::new(RandomStrategy::default()) as Box<dyn types::Strategy>,
        ),
    ]);

    let mut game_state2 = GameState::new(player_inputs_2);
    run_game(&mut game_state2, None, &mut recorder, None)
        .await
        .expect("Second game should complete successfully");

    assert!(!game_state2.still_playing());
}
