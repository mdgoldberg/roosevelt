use std::fmt::Display;

use itertools::Itertools;
use uuid::Uuid;

use crate::{card::Card, card_play::CardPlay};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    SendCard { to: Uuid, card: Card },
    PlayCards { card_play: CardPlay },
    Pass,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Action::SendCard { card, .. } => format!("Send {card}"),
            Action::Pass => "Pass".to_string(),
            Action::PlayCards { card_play } => {
                let cards = card_play.cards().map(|card| format!("{card}")).join(",");
                format!("Play {cards}")
            }
        };
        write!(f, "{}", string)
    }
}

impl From<&CardPlay> for Action {
    fn from(card_play: &CardPlay) -> Self {
        Action::PlayCards {
            card_play: *card_play,
        }
    }
}

// impl PartialOrd for Action {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         match (self, other) {
//             (
//                 Action::SendCard { card, .. },
//                 Action::SendCard {
//                     card: other_card, ..
//                 },
//             ) => card.partial_cmp(other_card),
//             (Action::SendCard { .. }, Action::PlayCards { .. }) => Some(Ordering::Less),
//             (Action::Pass, Action::SendCard { .. }) => Some(Ordering::Greater),
//             (Action::Pass, Action::PlayCards { card_play }) => Some(Ordering::Greater),
//             (Action::PlayCards { card_play }, Action::SendCard { to, card }) => todo!(),
//             (Action::PlayCards { card_play }, Action::PlayCards { card_play }) => todo!(),
//
//             // Pass goes last
//             (Action::SendCard { to, card }, Action::Pass) => Some(Ordering::Less),
//             (Action::PlayCards { card_play }, Action::Pass) => Some(Ordering::Less),
//             (Action::Pass, Action::Pass) => Some(Ordering::Equal),
//         }
//     }
// }
