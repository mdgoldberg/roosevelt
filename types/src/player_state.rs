use std::fmt::{Debug, Display};

use itertools::Itertools;
use uuid::Uuid;

use crate::card::Card;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    President,
    VicePresident,
    Secretary,
    ViceAsshole,
    Asshole,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::President => write!(f, "President"),
            Role::VicePresident => write!(f, "VicePresident"),
            Role::Secretary => write!(f, "Secretary"),
            Role::ViceAsshole => write!(f, "ViceAsshole"),
            Role::Asshole => write!(f, "Asshole"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub id: Uuid,
    pub name: String,
    pub role: Option<Role>,
    pub current_hand: Vec<Card>,
}

#[derive(Clone, Debug)]
pub struct PublicPlayerState {
    pub id: Uuid,
    pub name: String,
    pub role: Option<Role>,
    pub hand_size: usize,
}

impl From<&PlayerState> for PublicPlayerState {
    fn from(value: &PlayerState) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            role: value.role,
            hand_size: value.current_hand.len(),
        }
    }
}

impl PartialEq for PlayerState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:?}) Hand: {}",
            self.name,
            self.role
                .map_or_else(|| "No Role".to_string(), |role| role.to_string()),
            self.current_hand.iter().sorted().join(", ")
        )
    }
}

impl PlayerState {
    pub fn new(name: String, dealt_hand: Vec<Card>, role: Option<Role>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            role,
            current_hand: dealt_hand,
        }
    }

    pub fn new_with_id(id: Uuid, name: String, dealt_hand: Vec<Card>, role: Option<Role>) -> Self {
        Self {
            id,
            name,
            role,
            current_hand: dealt_hand,
        }
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
