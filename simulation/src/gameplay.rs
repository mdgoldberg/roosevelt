use std::collections::VecDeque;

use deckofcards::{Deck, Rank, Suit};
use log;
use rand::seq::SliceRandom;
use rand::thread_rng;
use uuid::Uuid;

use crate::types::{Action, Card, Event, GameState, Player, Role};

impl GameState {
    pub fn new(player_names: &[&str]) -> Self {
        let mut deck = Deck::new();
        deck.reset_shuffle();
        let hand_size = deck.count() / player_names.len();
        log::info!(
            "Num players: {:?}, hand size: {hand_size:?}",
            player_names.len()
        );
        let mut players: Vec<_> = player_names
            .iter()
            .map(|name| {
                let cards: Vec<_> = deck.deal(hand_size).into_iter().map(|c| c.into()).collect();
                assert_eq!(cards.len(), hand_size);
                Player::new(name, cards, None)
            })
            .collect();

        players.shuffle(&mut thread_rng());
        let table = VecDeque::from(players);

        Self {
            table,
            top_card: None,
            history: Vec::new(),
        }
    }

    pub fn permitted_actions(&self) -> Vec<Action> {
        if !self.pregame_done() {
            self.pregame_actions()
        } else {
            self.ingame_actions()
        }
    }

    /// Pregame: Asshole, Asshole, President, President, ViceAsshole, VP
    fn pregame_done(&self) -> bool {
        todo!()
        // let has_roles = self.get_role(Role::President).is_some()
        //     || self.get_role(Role::VicePresident).is_some();
        // if !has_roles {
        //     return true;
        // }
    }

    /// Pregame: Asshole, Asshole, President, President, ViceAsshole, VP
    fn pregame_actions(&self) -> Vec<Action> {
        todo!()
        // let card_passes = self
        //     .history
        //     .iter()
        //     .filter(|event| matches!(event.action, Action::SendCard { .. }))
        //     .map(|event| event.player_id).collect();
    }

    fn ingame_actions(&self) -> Vec<Action> {
        todo!()
    }

    pub fn perform_action(&mut self, action: &Action) {
        let player = self.current_player_mut();
        let player_id = player.id;
        match action {
            Action::SendCard { to, card } => {
                let removed = player.current_hand.remove_card(&card);
                assert!(
                    removed,
                    "Attempted to send a card {:?} that wasn't in the hand!",
                    card
                );
                let receiving_player = self.get_player_mut(*to).expect("Player with ID must exist");
                receiving_player.current_hand.push(*card);
                // NOTE: not populating pregame_cards_sent or pregame_cards_received
            }
            Action::Pass => {}
            Action::PlayCards { cards } => {
                cards.iter().for_each(|card| {
                    let removed = player.current_hand.remove_card(card);
                    assert!(
                        removed,
                        "Attempted to play a card {:?} that wasn't in the hand!",
                        card
                    );
                });
                // check that played cards are greater than top card
                assert!(
                    cards.get(0).map(|c| c.rank().ordinal())
                        > self
                            .top_card
                            .as_ref()
                            .and_then(|cards| cards.get(0))
                            .map(|c| c.rank().ordinal())
                );
                self.top_card = Some(cards.clone());
            }
        }
        // record event in history
        let event = Event {
            player_id,
            action: action.clone(),
        };
        self.history.push(event);

        // next player's turn
        self.table.rotate_left(1);

        // TODO: handle clearing the deck
    }

    pub fn run_pregame(&mut self) {
        self.swap_cards_by_role(Role::Asshole, Role::President, 2);
        self.swap_cards_by_role(Role::ViceAsshole, Role::VicePresident, 1);
        self.set_starting_player();
    }

    pub fn get_player(&mut self, id: Uuid) -> Option<&Player> {
        self.table.iter().find(|p| p.id == id)
    }

    pub fn get_role(&self, role: Role) -> Option<&Player> {
        self.table.iter().find(|p| p.role == Some(role))
    }

    pub fn current_player(&self) -> &Player {
        self.table
            .front()
            .expect("Should always have a current player")
    }

    fn get_player_mut(&mut self, id: Uuid) -> Option<&mut Player> {
        self.table.iter_mut().find(|p| p.id == id)
    }

    fn get_role_mut(&mut self, role: Role) -> Option<&mut Player> {
        self.table.iter_mut().find(|p| p.role == Some(role))
    }

    fn current_player_mut(&mut self) -> &mut Player {
        self.table
            .front_mut()
            .expect("Should always have a current player")
    }

    fn set_starting_player(&mut self) {
        let mut starter_id: Option<Uuid> = None;
        for three_card in [
            Card::new(Rank::Three, Suit::Clubs),
            Card::new(Rank::Three, Suit::Spades),
            Card::new(Rank::Three, Suit::Hearts),
            Card::new(Rank::Three, Suit::Diamonds),
            Card::new(Rank::Four, Suit::Clubs),
        ] {
            if let Some(starter) = self
                .table
                .iter()
                .find(|player| player.current_hand.contains(&three_card))
            {
                starter_id = Some(starter.id);
                break;
            }
        }
        let starter_id = starter_id.expect("Someone must have one of: 3C, 3S, 3H, 3D, 4S");
        let idx = self
            .table
            .iter()
            .position(|p| p.id == starter_id)
            .expect("Someone must have one of these cards");
        self.table.rotate_left(idx);
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
            .map(|_| {
                asshole
                    .pop_top_card()
                    .unwrap_or_else(|| panic!("Should always have {num_cards}+ cards in pregame"))
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
            .map(|_| {
                president
                    .pop_bottom_card()
                    .unwrap_or_else(|| panic!("Should always have {num_cards}+ cards in pregame"))
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

    pub fn still_playing(&self) -> bool {
        self.table
            .iter()
            .filter(|player| !player.current_hand.is_empty())
            .count()
            >= 2
    }

    pub fn start_new_game(&mut self) {
        // TODO: scan history to assign new roles for next game
        // TODO: should enable option to shuffle seating order between games
        todo!()
    }
}

impl Player {
    pub fn new(name: &str, dealt_hand: Vec<Card>, role: Option<Role>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            role,
            pregame_cards_sent: vec![],
            pregame_cards_received: vec![],
            current_hand: dealt_hand,
        }
    }

    pub fn select_action<'a>(
        &self,
        game_state: &GameState,
        available_actions: &[Action],
    ) -> Action {
        todo!()
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

trait Hand {
    fn remove_card(&mut self, card: &Card) -> bool;
}

impl Hand for Vec<Card> {
    fn remove_card(&mut self, card: &Card) -> bool {
        if let Some(idx) = self.iter().position(|c| c == card) {
            self.swap_remove(idx);
            true
        } else {
            false
        }
    }
}
