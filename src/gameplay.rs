use deckofcards::Deck;
use itertools::Itertools;
// use itertools::Itertools;
use log;

use crate::types::{Action, Card, GameState, Player, Role};

impl GameState {
    pub fn new(player_names: &[&str]) -> Self {
        let mut deck = Deck::new();
        deck.reset_shuffle();
        let hand_size = deck.count() / player_names.len();
        log::info!(
            "Num players: {:?}, hand size: {hand_size:?}",
            player_names.len()
        );
        let players: Vec<_> = player_names
            .iter()
            .map(|name| {
                let cards: Vec<_> = deck.deal(hand_size).into_iter().map(|c| c.into()).collect();
                assert_eq!(cards.len(), hand_size);
                Player::new(name, cards, None)
            })
            .collect();
        Self {
            players,
            whose_turn: 0,  // TODO
            top_card: None, // TODO
            history: Vec::new(),
        }
    }

    pub fn get_role(&mut self, role: Role) -> Option<&Player> {
        self.players.iter().find(|p| p.role == Some(role))
    }

    pub fn get_role_mut(&mut self, role: Role) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.role == Some(role))
    }

    pub fn current_player(&mut self) -> &mut Player {
        self.players
            .get_mut(self.whose_turn)
            .expect("whose_turn should always be < len(players)")
    }

    pub fn next_turn(&mut self) {
        self.whose_turn = (self.whose_turn + 1) % self.players.len();
    }

    pub fn perform_turn(&mut self, action: &Action) {
        let current_player = self.current_player();
        match &action {
            Action::Pass => todo!(),
            Action::Play(_) => todo!(),
        }
    }

    pub fn run_pregame(&mut self) {
        self.swap_cards_by_role(Role::Asshole, Role::President, 2);
        self.swap_cards_by_role(Role::ViceAsshole, Role::VicePresident, 1);
    }

    fn swap_cards_by_role(&mut self, asshole_role: Role, president_role: Role, num_cards: usize) {
        match (
            self.get_role(asshole_role).is_some(),
            self.get_role(president_role).is_some(),
        ) {
            (false, false) => {
                log::warn!("No players found for either role, so not swapping any cards: {asshole_role:?}, {president_role:?}");
                return;
            }
            (false, true) => {
                log::error!("Found player for {president_role:?} but not for {asshole_role:?}, so not swapping cards");
                return;
            }
            (true, false) => {
                log::error!("Found player for {president_role:?} but not for {asshole_role:?}, so not swapping cards");
                return;
            }
            (true, true) => (),
        };

        let asshole = self
            .get_role_mut(asshole_role)
            .expect("Should have already checked for asshole existence");
        let asshole_top_cards: Vec<_> = (0..num_cards)
            .into_iter()
            .map(|_| {
                asshole
                    .pop_top_card()
                    .expect(format!("Should always have {num_cards}+ cards in pregame").as_str())
            })
            .collect();
        asshole
            .pregame_cards_sent
            .extend(&mut asshole_top_cards.iter().copied());

        let president = self
            .get_role_mut(president_role)
            .expect("Should have already checked for president existence");

        // TODO: president/VP should send bottom cards strategically
        let president_sending_cards: Vec<_> = (0..num_cards)
            .into_iter()
            .map(|_| {
                president
                    .pop_bottom_card()
                    .expect(format!("Should always have {num_cards}+ cards in pregame").as_str())
            })
            .collect();

        president
            .pregame_cards_received
            .extend(&mut asshole_top_cards.iter().copied());
        president
            .pregame_cards_sent
            .extend(&mut president_sending_cards.iter().copied());

        // reaquire mutable reference to asshole to set their received cards
        let asshole = self
            .get_role_mut(asshole_role)
            .expect("Should have already checked for asshole existence");
        asshole
            .pregame_cards_received
            .extend(&mut president_sending_cards.iter().copied());
    }
}

impl Player {
    pub fn new(name: &str, dealt_hand: Vec<Card>, role: Option<Role>) -> Self {
        Self {
            name: name.to_string(),
            role,
            pregame_cards_sent: vec![],
            pregame_cards_received: vec![],
            current_hand: dealt_hand,
        }
    }

    pub fn pop_top_card(&mut self) -> Option<Card> {
        let top_card = self.current_hand.iter().max()?;
        let index = self
            .current_hand
            .iter()
            .position(|c| c == top_card)
            .expect("Top card will be in hand");
        Some(self.current_hand.remove(index))
    }

    pub fn pop_bottom_card(&mut self) -> Option<Card> {
        let bottom_card = self.current_hand.iter().min()?;
        let index = self
            .current_hand
            .iter()
            .position(|c| c == bottom_card)
            .expect("Top card will be in hand");
        Some(self.current_hand.remove(index))
    }
}
