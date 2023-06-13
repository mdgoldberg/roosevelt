use crate::types::{Action, Card, CardPlay};

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
}

impl From<&CardPlay> for Action {
    fn from(&card_play: &CardPlay) -> Self {
        Action::PlayCards { card_play }
    }
}
