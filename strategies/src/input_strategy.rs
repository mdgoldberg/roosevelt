use std::io::{self, Write};

use deckofcards::{Rank, Suit};
use itertools::Itertools;
use regex::{Captures, Regex};
use types::{game_state::PublicInfo, Action, PlayerState, Strategy};

#[derive(Debug, Default)]
pub struct InputStrategy {}

impl Strategy for InputStrategy {
    fn select_action(
        &mut self,
        private_info: &PlayerState,
        public_info: &PublicInfo,
        available_actions: &[Action],
    ) -> Action {
        print_public_info(&public_info);
        println!("Private info: {private_info}");
        println!(
            "Available actions: {}",
            available_actions.iter().sorted().join(" || ")
        );

        // if only one available action, do it
        if available_actions.len() == 1 {
            let action = *available_actions
                .get(0)
                .expect("Guaranteed to have an action available");
            log::info!("Only have one action available: {action}");
            return action;
        }

        let mut buf = String::new();
        loop {
            match select_action_from_stdin(&mut buf, available_actions) {
                Ok(action) => return action,
                Err(err) => {
                    buf.clear();
                    log::error!("Error parsing message from stdin: {err}")
                }
            }
        }
    }
}

fn print_public_info(info: &PublicInfo) {
    for player_pub_info in info.public_table.iter() {
        let card_plays: Vec<_> = info
            .history
            .iter()
            .filter_map(|event| {
                if event.player_id != player_pub_info.id {
                    return None;
                }
                match event.action {
                    Action::PlayCards { card_play } => Some(card_play),
                    _ => return None,
                }
            })
            .collect();
        println!(
            "{} ({}) has {} cards left and has played: [ {} ]",
            player_pub_info.name,
            player_pub_info
                .role
                .map(|r| r.to_string())
                .unwrap_or("None".to_string()),
            player_pub_info.hand_size,
            card_plays.iter().sorted().join(", ")
        );
    }
    println!(
        "Top card is: {}",
        info.top_card
            .map(|cp| cp.to_string())
            .unwrap_or("None".to_string())
    );
}

fn select_action_from_stdin(buf: &mut String, actions: &[Action]) -> Result<Action, String> {
    print!("Your action? >> ");
    let _ = io::stdout().flush();
    match io::stdin().read_line(buf) {
        Ok(_) => select_action_from_str(&buf, actions),
        Err(err) => {
            buf.clear();
            Err(format!("Error reading line from stdin: {err}"))
        }
    }
}

fn select_action_from_str(input: &str, actions: &[Action]) -> Result<Action, String> {
    let input = input.to_lowercase();
    let input = input.as_str();

    if let Some(send_result) = _get_action_from_regex(
        input,
        actions,
        Regex::new(r"send (?<card>\S+)").expect("Valid send regex"),
        send_action_from_captures,
    ) {
        return send_result;
    }

    if let Some(pass_result) = _get_action_from_regex(
        input,
        actions,
        Regex::new(r"pass").expect("Valid pass regex"),
        pass_action_from_captures,
    ) {
        return pass_result;
    }

    if let Some(play_result) = _get_action_from_regex(
        input,
        actions,
        Regex::new(r"play (.+)").expect("Valid play regex"),
        play_action_from_captures,
    ) {
        return play_result;
    }

    Err(format!(
        "Unable to parse a permitted Send, Pass, or Play action from string: {input}"
    ))
}

fn _get_action_from_regex(
    input: &str,
    actions: &[Action],
    re: Regex,
    callback: for<'a, 'b, 'c> fn(&'a regex::Captures<'b>, &'c [Action]) -> Result<Action, String>,
) -> Option<Result<Action, String>> {
    let Some(caps) = re.captures(input) else {
        return None;
    };

    Some(callback(&caps, actions))
}

fn send_action_from_captures(caps: &Captures, actions: &[Action]) -> Result<Action, String> {
    let (rank, suit) = parse_card(
        caps.name("card")
            .expect("Send match always has card")
            .as_str(),
    )?;
    actions
        .iter()
        .filter(|act| {
            if let Action::SendCard { card, .. } = act {
                card.rank() == rank && suit.map(|s| s == card.suit()).unwrap_or(true)
            } else {
                false
            }
        })
        .next()
        .copied()
        .ok_or_else(|| format!("No card matching {rank:?}"))
}

fn pass_action_from_captures(_caps: &Captures, actions: &[Action]) -> Result<Action, String> {
    if actions.iter().any(|act| *act == Action::Pass) {
        Ok(Action::Pass)
    } else {
        Err("Attempted to pass at a time when passing is not a permitted action".to_string())
    }
}

fn play_action_from_captures(caps: &Captures, actions: &[Action]) -> Result<Action, String> {
    log::debug!("Captured: {caps:?}");
    let input = caps.get(0).expect("Group 0 is guaranteed").as_str();
    let mut play_str = caps
        .get(1)
        .expect("Group 1 is required in play regex")
        .as_str();

    let muncher_re =
        Regex::new(r"(?<card>\w+)(?:[,\s]\s*(?<tail>.*))?").expect("Valid play muncher regex");
    let mut cards = Vec::new();
    while let Some(caps) = muncher_re.captures(play_str) {
        log::debug!("Captured: {caps:?}");
        let card = parse_card(
            caps.name("card")
                .expect("card is a required group")
                .as_str(),
        )?;
        play_str = caps.name("tail").map(|m| m.as_str()).unwrap_or("");
        cards.push(card);
    }

    if cards.is_empty() {
        return Err(format!(
            "Attempted to play cards, but no cards found in string: {input:?}"
        ));
    }

    let num_unique_ranks = cards.iter().unique_by(|c| c.0).count();
    if num_unique_ranks > 1 {
        return Err(format!(
            "Attempted to play multiple cards that aren't the same rank: {:?}",
            cards.iter().map(|c| c.0).collect::<Vec<_>>()
        ));
    }

    let suit_counts = cards.iter().filter_map(|c| c.1).counts_by(|c| c);
    if let Some((mode_suit, mode_suit_count)) = suit_counts.iter().max_by_key(|s| s.1) {
        if *mode_suit_count > 1 {
            let rank = cards.get(0).expect("Guaranteed to have at least 1 card").0;
            return Err(format!(
                    "Attempted to play multiple cards of rank {rank:?}, but {mode_suit_count} are the same suit {mode_suit:?}"
                    ));
        }
    }

    let rank = cards.get(0).expect("Guaranteed at least one card").0;
    let &card_play = actions
        .iter()
        .filter_map(|act| {
            if let Action::PlayCards { card_play } = act {
                Some(card_play)
            } else {
                None
            }
        })
        // correct # of cards?
        .filter(|cp| cp.size() == cards.len())
        // rank matches and all suits are accounted for?
        .filter(|cp| {
            let cp_cards = cp.to_vec();
            let suits: Vec<Suit> = cards.iter().filter_map(|c| c.1).collect();
            cp.rank() == rank
                && suits
                    .iter()
                    .all(|suit| cp_cards.iter().any(|c| c.suit() == *suit))
        })
        .next()
        .ok_or_else(|| format!("Unable to find a permitted action matching the input string"))?;

    log::debug!("From actions {actions:?}, given string {input:?}, selected {card_play:?}");

    return Ok(Action::PlayCards { card_play });
}

fn parse_card(card_str: &str) -> Result<(Rank, Option<Suit>), String> {
    let card_re = Regex::new(r"(?<rank>\w)(?<suit>\S)?").expect("Valid send regex");
    let Some(caps) = card_re.captures(card_str) else {
        return Err(format!("Unable to parse card from {card_str}"));
    };
    let rank = rank_from_rank_str(
        &caps
            .name("rank")
            .expect("Rank should always exist for send")
            .as_str(),
    )?;
    let suit = caps
        .name("suit")
        .map(|m| suit_from_suit_str(m.as_str()))
        .map_or(Ok(None), |r| r.map(Some))?;
    Ok((rank, suit))
}

fn rank_from_rank_str(s: &str) -> Result<Rank, String> {
    match s.to_uppercase().as_str() {
        "2" => Ok(Rank::Two),
        "3" => Ok(Rank::Three),
        "4" => Ok(Rank::Four),
        "5" => Ok(Rank::Five),
        "6" => Ok(Rank::Six),
        "7" => Ok(Rank::Seven),
        "8" => Ok(Rank::Eight),
        "9" => Ok(Rank::Nine),
        "T" => Ok(Rank::Ten),
        "J" => Ok(Rank::Jack),
        "Q" => Ok(Rank::Queen),
        "K" => Ok(Rank::King),
        "A" => Ok(Rank::Ace),
        _ => Err(format!("Unable to convert string to rank: {s}")),
    }
}

fn suit_from_suit_str(s: &str) -> Result<Suit, String> {
    match s.to_lowercase().as_str() {
        "s" | "\u{2660}" => Ok(Suit::Spades),
        "h" | "\u{2665}" => Ok(Suit::Hearts),
        "d" | "\u{2666}" => Ok(Suit::Diamonds),
        "c" | "\u{2663}" => Ok(Suit::Clubs),
        _ => Err(format!("Unable to convert string to suit: {s}")),
    }
}
