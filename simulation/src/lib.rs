use database::{DatabaseWriter, GameMetadata};
use std::{thread::sleep, time::Duration};
use types::game_state::GameState;
use types::{Action, Player};

pub async fn run_game(
    game_state: &mut GameState,
    delay_ms: Option<u64>,
    recorder: &mut dyn DatabaseWriter,
    game_config: Option<serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let seed = generate_deck_seed();
    let player_order: Vec<_> = game_state.table.iter().map(|p| p.state.id).collect();

    // Record players first
    for player in &game_state.table {
        recorder
            .record_player(player.state.id, &format!("Player_{}", player.state.id))
            .await?;
    }

    let game_meta = GameMetadata {
        started_at: chrono::Utc::now(),
        num_players: game_state.table.len(),
        deck_seed: format!("{:x}", seed),
        player_order,
        configuration: game_config,
    };

    let handle = recorder.start_game(game_meta).await?;

    let pregame_events = game_state.run_pregame();

    let mut turn_order = 0;
    for event in &pregame_events {
        let (action_type, card_play_json, target_player_id) = match &event.action {
            Action::SendCard { card, to } => (
                "SendCard".to_string(),
                Some(serialize_card_play_single(card)),
                Some(to),
            ),
            _ => (action_type_to_string(&event.action), None, None),
        };

        recorder
            .record_action(
                handle,
                &database::models::ActionRecord {
                    id: None,
                    game_id: handle.as_i64(),
                    player_id: event.player_id,
                    action_type,
                    card_play: card_play_json.map(|v| serde_json::to_value(&v).unwrap()),
                    target_player_id: target_player_id.copied(),
                    turn_order: turn_order + 1,
                    phase: "pregame".to_string(),
                    created_at: chrono::Utc::now(),
                },
            )
            .await?;

        turn_order += 1;
    }

    while game_state.still_playing() {
        log::debug!("{game_state}");
        if let Some(ms) = delay_ms {
            sleep(Duration::from_millis(ms));
        }
        let available_actions = game_state.permitted_actions();
        let public_info = game_state.public_info();
        let current_player = game_state.current_player_mut();
        let selected_action = current_player.strategy.select_action(
            &current_player.state,
            &public_info,
            &available_actions,
        );

        let (action_type, card_play_json, target_player_id) = match &selected_action {
            Action::PlayCards { card_play } => (
                "PlayCards".to_string(),
                Some(serialize_card_play(card_play)),
                None,
            ),
            Action::SendCard { card, to } => (
                "SendCard".to_string(),
                Some(serialize_card_play_single(card)),
                Some(to),
            ),
            Action::Pass => ("Pass".to_string(), None, None),
        };

        recorder
            .record_action(
                handle,
                &database::models::ActionRecord {
                    id: None,
                    game_id: handle.as_i64(),
                    player_id: current_player.state.id,
                    action_type,
                    card_play: card_play_json.map(|v| serde_json::to_value(&v).unwrap()),
                    target_player_id: target_player_id.copied(),
                    turn_order: turn_order + 1,
                    phase: "ingame".to_string(),
                    created_at: chrono::Utc::now(),
                },
            )
            .await?;

        game_state.perform_ingame_action(&selected_action);
        turn_order += 1;
    }

    let players_in_finishing_order = get_players_in_finishing_order(game_state);

    let results: Vec<database::models::GameResultRecord> = players_in_finishing_order
        .iter()
        .enumerate()
        .map(|(place, player)| {
            let finishing_place = place + 1;
            let finishing_role = calculate_role(finishing_place, game_state.table.len());

            database::models::GameResultRecord {
                id: None,
                game_id: handle.as_i64(),
                player_id: player.state.id,
                finishing_place,
                finishing_role,
            }
        })
        .collect();

    recorder.finish_game(handle, &results).await?;

    Ok(())
}

fn action_type_to_string(action: &Action) -> String {
    match action {
        Action::SendCard { .. } => "SendCard".to_string(),
        Action::PlayCards { .. } => "PlayCards".to_string(),
        Action::Pass => "Pass".to_string(),
    }
}

fn serialize_card_play(card_play: &types::CardPlay) -> serde_json::Value {
    match card_play {
        types::CardPlay::Single(card) => serde_json::json!({
            "type": "Single",
            "cards": [card.to_string()]
        }),
        types::CardPlay::Pair(c1, c2) => serde_json::json!({
            "type": "Pair",
            "cards": [c1.to_string(), c2.to_string()]
        }),
        types::CardPlay::Triple(c1, c2, c3) => serde_json::json!({
            "type": "Triple",
            "cards": [c1.to_string(), c2.to_string(), c3.to_string()]
        }),
        types::CardPlay::Quad(c1, c2, c3, c4) => serde_json::json!({
            "type": "Quad",
            "cards": [c1.to_string(), c2.to_string(), c3.to_string(), c4.to_string()]
        }),
    }
}

fn serialize_card_play_single(card: &types::Card) -> serde_json::Value {
    serde_json::json!({
        "type": "Single",
        "cards": [card.to_string()]
    })
}

fn calculate_role(finishing_place: usize, num_players: usize) -> String {
    match finishing_place {
        1 => "President".to_string(),
        2 => "VicePresident".to_string(),
        place if place == num_players - 1 => "ViceAsshole".to_string(),
        place if place == num_players => "Asshole".to_string(),
        _ => "Secretary".to_string(),
    }
}

fn get_players_in_finishing_order(game_state: &GameState) -> Vec<&Player> {
    let mut worst_to_first = Vec::new();

    for player in &game_state.table {
        if !player.state.current_hand.is_empty() {
            worst_to_first.push(player);
        }
    }

    for event in game_state.history.iter().rev() {
        if matches!(event.action, Action::PlayCards { .. })
            && !worst_to_first.iter().any(|p| p.state.id == event.player_id)
        {
            if let Some(player) = game_state.get_player(event.player_id) {
                worst_to_first.push(player);
            }
        }
    }

    worst_to_first.into_iter().rev().collect()
}

pub fn generate_deck_seed() -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen()
}
