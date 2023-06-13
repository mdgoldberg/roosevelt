use std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::{Debug, Display},
};

use deckofcards::{Card as DOCCard, Rank, Suit};
use itertools::Itertools;
use uuid::Uuid;

#[derive(Debug)]
pub struct GameState {
    pub table: VecDeque<Player>,
    pub top_card: Option<CardPlay>,
    pub history: Vec<Event>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top_card_str = self
            .top_card
            .map(|card_play| format!("{card_play}"))
            .unwrap_or("None".to_string());
        let players_str = self
            .table
            .iter()
            .map(|player| {
                format!(
                    "{}: {} cards left: {}",
                    player.name,
                    player.current_hand.len(),
                    player
                        .current_hand
                        .iter()
                        .sorted()
                        .map(|c| format!("{c}"))
                        .join(",")
                )
            })
            .join("\n");
        write!(f, "\nTop Card: {}\nTable:\n{}", top_card_str, players_str)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub player_id: Uuid,
    pub action: Action,
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    SendCard { to: Uuid, card: Card },
    Pass,
    PlayCards { card_play: CardPlay },
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Action::SendCard { card, .. } => format!("Send {card}"),
            Action::Pass => "Pass".to_string(),
            Action::PlayCards { card_play } => {
                format!("Play {}", card_play.to_vec().iter().join(","))
            }
        };
        write!(f, "{}", string)
    }
}

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
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
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
        our_card.partial_cmp(their_card)
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub role: Option<Role>,
    pub current_hand: Vec<Card>,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    President,
    VicePresident,
    Secretary,
    ViceAsshole,
    Asshole,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    card: DOCCard,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.card)
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
