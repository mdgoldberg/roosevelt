use deckofcards::{Rank, Suit};
use itertools::Itertools;
use uuid::Uuid;

use crate::types::{Action, Card, GameState, Player, Role};

impl Player {
    pub fn new(name: &str, dealt_hand: Vec<Card>, role: Option<Role>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            role,
            current_hand: dealt_hand,
        }
    }

    pub fn select_action<'a>(
        &self,
        _game_state: &GameState,
        available_actions: &[Action],
    ) -> Action {
        // always play worst allowable card play
        if let Some(card_play_action) = available_actions
            .iter()
            .filter_map(|action| {
                if let Action::PlayCards { card_play } = action {
                    Some((action, card_play))
                } else {
                    None
                }
            })
            .min_by_key(|(_, cp)| (cp.size(), cp.value()))
            .map(|(action, _)| action)
        {
            return *card_play_action;
        }

        // always send worst card
        if let Some(pass_card_action) = available_actions
            .iter()
            .filter_map(|action| {
                if let Action::SendCard { card, .. } = action {
                    let is_three_of_clubs =
                        card.rank() == Rank::Three && card.suit() == Suit::Clubs;
                    if is_three_of_clubs {
                        None
                    } else {
                        Some((action, card))
                    }
                } else {
                    None
                }
            })
            .min_by_key(|(_, card)| card.value())
            .map(|(action, _)| action)
        {
            return *pass_card_action;
        }

        available_actions[0]
    }

    pub fn top_k_cards(&self, num_cards: usize) -> Vec<Card> {
        self.current_hand
            .iter()
            .sorted()
            .rev()
            .take(num_cards)
            .copied()
            .collect()
    }

    pub fn bottom_k_cards(&self, num_cards: usize) -> Vec<Card> {
        self.current_hand
            .iter()
            .sorted()
            .take(num_cards)
            .copied()
            .collect()
    }
}
