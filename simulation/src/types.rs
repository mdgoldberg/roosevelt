use std::{cmp::Ordering, collections::VecDeque, fmt::Debug};

use deckofcards::{Card as DOCCard, Rank, Suit};
use uuid::Uuid;

#[derive(Debug)]
pub struct GameState {
    pub table: VecDeque<Player>,
    pub top_card: Option<Vec<Card>>,
    pub history: Vec<Event>,
}

#[derive(Debug)]
pub struct Event {
    pub player_id: Uuid,
    pub action: Action,
}

#[derive(Clone, Debug)]
pub enum Action {
    SendCard { to: Uuid, card: Card },
    Pass,
    PlayCards { cards: Vec<Card> },
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub pregame_cards_received: Vec<Card>,
    pub pregame_cards_sent: Vec<Card>,
    pub role: Option<Role>,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Card {
    card: DOCCard,
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
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.card.rank == other.card.rank {
            return Some(Ordering::Equal);
        }
        if self.card.rank == Rank::Two {
            return Some(Ordering::Greater);
        }
        if other.card.rank == Rank::Two {
            return Some(Ordering::Less);
        }
        Some(self.card.rank.ordinal().cmp(&other.card.rank.ordinal()))
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
