pub mod input_strategy;

use deckofcards::{Rank, Suit};
use rand::{rngs::ThreadRng, seq::SliceRandom};
use types::{Action, Strategy};

pub use crate::input_strategy::InputStrategy;

#[derive(Debug, Default)]
pub struct RandomStrategy {
    rng: ThreadRng,
}

impl Strategy for RandomStrategy {
    fn select_action(
        &mut self,
        _private_info: &types::PlayerState,
        _public_info: &types::game_state::PublicInfo,
        available_actions: &[Action],
    ) -> Action {
        *available_actions
            .choose(&mut self.rng)
            .expect("Should always have at least one action to choose from")
    }
}

#[derive(Debug, Default)]
pub struct DefaultStrategy {}

impl Strategy for DefaultStrategy {
    fn select_action(
        &mut self,
        _private_info: &types::PlayerState,
        _public_info: &types::game_state::PublicInfo,
        available_actions: &[types::Action],
    ) -> types::Action {
        // always play worst allowable card play
        // TODO: play low sets when there's no top card
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
            .filter_map(|action| -> Option<(&Action, &types::Card)> {
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

        *available_actions
            .first()
            .expect("Always should have an action available when this is called")
    }
}
