use itertools::Itertools;

use crate::types::{Card, CardPlay};

pub trait Hand {
    fn remove_card(&mut self, card: &Card) -> bool;
    fn singles(&self) -> Vec<CardPlay>;
    fn pairs(&self) -> Vec<CardPlay>;
    fn triples(&self) -> Vec<CardPlay>;
    fn quads(&self) -> Vec<CardPlay>;
}

fn _card_plays_for_size(hand: &[Card], card_play_size: usize) -> Vec<CardPlay> {
    hand.iter()
        .combinations(card_play_size)
        .filter_map(|cards| {
            let rank = cards.get(0)?.rank();
            if cards.iter().all(|c| c.rank() == rank) {
                Some(CardPlay::from_cards(&cards))
            } else {
                None
            }
        })
        .collect()
}

impl Hand for Vec<Card> {
    fn remove_card(&mut self, card: &Card) -> bool {
        if let Some(idx) = self.iter().position(|c| c == card) {
            self.swap_remove(idx);
            true
        } else {
            false
        }
    }

    fn singles(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 1)
    }

    fn pairs(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 2)
    }

    fn triples(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 3)
    }

    fn quads(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 4)
    }
}
