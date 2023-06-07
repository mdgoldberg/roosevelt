use std::{cmp::Ordering, fmt::Debug};

use deckofcards::{Card as DOCCard, Rank};

#[derive(Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub whose_turn: usize,
    pub top_card: Option<Vec<Card>>,
    pub history: Vec<Action>,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub name: String,
    pub role: Option<Role>,
    pub pregame_cards_received: Vec<Card>,
    pub pregame_cards_sent: Vec<Card>,
    pub current_hand: Vec<Card>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Role {
    President,
    VicePresident,
    Secretary,
    ViceAsshole,
    Asshole,
}

#[derive(Debug)]
pub enum Action {
    Pass,
    Play(Vec<Card>),
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct Card {
    card: DOCCard,
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
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
