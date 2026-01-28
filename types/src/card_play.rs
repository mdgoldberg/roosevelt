use std::{cmp::Ordering, fmt::Display, iter::FusedIterator};

use deckofcards::Rank;
use itertools::Itertools;

use crate::card::Card;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CardPlay {
    Single(Card),
    Pair(Card, Card),
    Triple(Card, Card, Card),
    Quad(Card, Card, Card, Card),
}

impl Display for CardPlay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.cards().map(|card| format!("{}", card)).join(", ")
        )
    }
}

impl PartialOrd for CardPlay {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardPlay {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by size first ( Singles < Pairs < Triples < Quads )
        self.size()
            .cmp(&other.size())
            .then_with(|| self.value().cmp(&other.value()))
    }
}

impl CardPlay {
    pub fn from_cards(cards: &[&Card]) -> CardPlay {
        match *cards {
            [card] => CardPlay::Single(*card),
            [card1, card2] => CardPlay::Pair(*card1, *card2),
            [card1, card2, card3] => CardPlay::Triple(*card1, *card2, *card3),
            [card1, card2, card3, card4] => CardPlay::Quad(*card1, *card2, *card3, *card4),
            _ => panic!(
                "Attempted to make a CardPlay from more than four cards: {:?}",
                cards
            ),
        }
    }
    pub fn cards(&self) -> CardPlayCards<'_> {
        CardPlayCards {
            card_play: self,
            idx: 0,
        }
    }

    pub fn to_vec(&self) -> Vec<Card> {
        self.cards().collect()
    }

    pub fn size(&self) -> usize {
        match self {
            CardPlay::Single(_) => 1,
            CardPlay::Pair(_, _) => 2,
            CardPlay::Triple(_, _, _) => 3,
            CardPlay::Quad(_, _, _, _) => 4,
        }
    }

    pub fn value(&self) -> usize {
        match self {
            CardPlay::Single(card) => card.value(),
            CardPlay::Pair(card, _) => card.value(),
            CardPlay::Triple(card, _, _) => card.value(),
            CardPlay::Quad(card, _, _, _) => card.value(),
        }
    }

    pub fn rank(&self) -> Rank {
        match self {
            CardPlay::Single(card) => card.rank(),
            CardPlay::Pair(card, _) => card.rank(),
            CardPlay::Triple(card, _, _) => card.rank(),
            CardPlay::Quad(card, _, _, _) => card.rank(),
        }
    }
}

pub struct CardPlayCards<'a> {
    card_play: &'a CardPlay,
    idx: usize,
}

impl<'a> Iterator for CardPlayCards<'a> {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        let card = match (self.card_play, self.idx) {
            (CardPlay::Single(card), 0) => Some(*card),
            (CardPlay::Pair(card1, card2), idx) => match idx {
                0 => Some(*card1),
                1 => Some(*card2),
                _ => None,
            },
            (CardPlay::Triple(card1, card2, card3), idx) => match idx {
                0 => Some(*card1),
                1 => Some(*card2),
                2 => Some(*card3),
                _ => None,
            },
            (CardPlay::Quad(card1, card2, card3, card4), idx) => match idx {
                0 => Some(*card1),
                1 => Some(*card2),
                2 => Some(*card3),
                3 => Some(*card4),
                _ => None,
            },
            _ => None,
        }?;
        self.idx += 1;
        Some(card)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.card_play.size().saturating_sub(self.idx);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for CardPlayCards<'a> {}

impl<'a> FusedIterator for CardPlayCards<'a> {}
