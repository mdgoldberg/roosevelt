use std::{cmp::Ordering, fmt::Display};

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
            self.to_vec()
                .iter()
                .map(|card| format!("{}", card))
                .join(", ")
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
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            panic!("Cannot compare CardPlay variants of different types");
        }
        let our_card = match self {
            CardPlay::Single(card) => card,
            CardPlay::Pair(card, _) => card,
            CardPlay::Triple(card, _, _) => card,
            CardPlay::Quad(card, _, _, _) => card,
        };
        let their_card = match other {
            CardPlay::Single(card) => card,
            CardPlay::Pair(card, _) => card,
            CardPlay::Triple(card, _, _) => card,
            CardPlay::Quad(card, _, _, _) => card,
        };
        our_card.cmp(their_card)
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
    pub fn to_vec(self: &CardPlay) -> Vec<Card> {
        match self {
            CardPlay::Single(card) => vec![*card],
            CardPlay::Pair(card1, card2) => vec![*card1, *card2],
            CardPlay::Triple(card1, card2, card3) => vec![*card1, *card2, *card3],
            CardPlay::Quad(card1, card2, card3, card4) => vec![*card1, *card2, *card3, *card4],
        }
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
