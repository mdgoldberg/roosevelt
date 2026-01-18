use std::{cmp::Ordering, fmt::Display};

use deckofcards::{Card as DOCCard, Rank, Suit};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    card: DOCCard,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suit_str = match self.card.suit {
            Suit::Spades => "\u{2660}",
            Suit::Hearts => "\u{2665}",
            Suit::Diamonds => "\u{2666}",
            Suit::Clubs => "\u{2663}",
        };
        write!(f, "{}{}", self.card.rank.to_char(), suit_str)
    }
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Card {
            card: DOCCard { rank, suit },
        }
    }

    pub fn all_cards() -> Vec<Card> {
        DOCCard::all_cards()
            .iter()
            .map(|&card| Card { card })
            .collect()
    }

    pub fn rank(&self) -> Rank {
        self.card.rank
    }

    pub fn suit(&self) -> Suit {
        self.card.suit
    }

    pub fn value(&self) -> usize {
        match self.rank() {
            Rank::Two => Rank::Ace.ordinal() + 1,
            rank => rank.ordinal(),
        }
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.card.rank == other.card.rank {
            return Ordering::Equal;
        }
        if self.card.rank == Rank::Two {
            return Ordering::Greater;
        }
        if other.card.rank == Rank::Two {
            return Ordering::Less;
        }
        self.card.rank.ordinal().cmp(&other.card.rank.ordinal())
    }
}

impl From<DOCCard> for Card {
    fn from(card: DOCCard) -> Self {
        Self { card }
    }
}
