use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

use deckofcards::{Deck, Rank, Suit};
use itertools::Itertools;
use log;
use rand::prelude::*;
use uuid::Uuid;

use crate::{
    action::Action, card_play::CardPlay, hand::Hand, player::Player, Card, PlayerState,
    PublicPlayerState, Role, Strategy,
};

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub player_id: Uuid,
    pub action: Action,
}

#[derive(Debug)]
pub struct GameState {
    pub table: VecDeque<Player>,
    pub top_card: Option<CardPlay>,
    pub history: Vec<Event>,
}

#[derive(Debug)]
pub struct PublicInfo {
    pub top_card: Option<CardPlay>,
    pub history: Vec<Event>,
    pub public_table: Vec<PublicPlayerState>,
}

impl GameState {
    pub fn new(player_inputs: Vec<(Uuid, String, Box<dyn Strategy>)>) -> Self {
        let num_players = player_inputs.len();
        let mut deck = Deck::new();
        deck.reset_shuffle();
        let hand_size = deck.count() / num_players;
        log::info!("Num players: {num_players:?}, hand size: {hand_size:?}");
        let mut players: Vec<_> = player_inputs
            .into_iter()
            .map(|(id, name, strat)| {
                let cards: Vec<_> = deck.deal(hand_size).into_iter().map_into().collect();
                assert_eq!(cards.len(), hand_size);
                Player {
                    state: PlayerState::new_with_id(id, name, cards, None),
                    strategy: strat,
                }
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

    pub fn public_info(&self) -> PublicInfo {
        PublicInfo {
            top_card: self.top_card,
            history: self.history.clone(),
            public_table: self.table.iter().map(|p| (&p.state).into()).collect(),
        }
    }

    pub fn permitted_actions(&self) -> Vec<Action> {
        let current_player = self.current_player();
        let hand = &current_player.state.current_hand;
        let mut actions: Vec<Action> = match self.top_card {
            None => [hand.singles(), hand.pairs(), hand.triples(), hand.quads()]
                .concat()
                .iter()
                .map_into()
                .collect(),
            Some(CardPlay::Single(..)) => hand
                .singles()
                .iter()
                .filter(|&&cp| Some(cp) > self.top_card)
                .map_into()
                .collect(),
            Some(CardPlay::Pair(..)) => hand
                .pairs()
                .iter()
                .filter(|&&cp| Some(cp) > self.top_card)
                .map_into()
                .collect(),
            Some(CardPlay::Triple(..)) => hand
                .triples()
                .iter()
                .filter(|&&cp| Some(cp) > self.top_card)
                .map_into()
                .collect(),
            Some(CardPlay::Quad(..)) => hand
                .quads()
                .iter()
                .filter(|&&cp| Some(cp) > self.top_card)
                .map_into()
                .collect(),
        };
        // allow passing if there's a card in play
        if self.top_card.is_some() {
            actions.push(Action::Pass);
        }
        // first card play must contain starting card
        let is_first_cardplay = !self
            .history
            .iter()
            .any(|ev| matches!(ev.action, Action::PlayCards { .. }));
        if is_first_cardplay {
            let (_, starting_card) = self.starting_player_and_card();
            actions.retain(|action| match action {
                Action::PlayCards { card_play } => {
                    card_play.cards().any(|card| card == starting_card)
                }
                _ => false,
            });
        }

        log::debug!(
            "Available actions for {}: {actions:?}",
            current_player.state.name
        );
        actions
    }

    pub fn perform_ingame_action(&mut self, action: &Action) {
        let player = self.current_player_mut();
        let player_id = player.state.id;
        match action {
            Action::SendCard { .. } => {
                panic!("Attempted to send a card in the middle of the game!");
            }
            Action::Pass => {}
            Action::PlayCards { card_play } => {
                for card in card_play.cards() {
                    let removed = player.state.current_hand.remove_card(&card);
                    assert!(
                        removed,
                        "Attempted to play a card {:?} that wasn't in the hand!",
                        card
                    );
                }
                // check that played cards are greater than top card
                assert!(Some(*card_play) > self.top_card);
                self.top_card = Some(*card_play);
            }
        }
        log::info!("{} did: {action}", self.current_player().state.name);
        // record event in history
        let event = Event {
            player_id,
            action: *action,
        };
        self.history.push(event);

        // also handles clearing the deck if necessary
        self.next_players_turn();
        while self.current_player().state.current_hand.is_empty() {
            self.next_players_turn();
        }
    }

    fn next_players_turn(&mut self) {
        self.table.rotate_left(1);

        // clear the deck if necessary
        if self.last_played_player().map(|p| &p.state) == Some(&self.current_player().state) {
            self.top_card = None;
        }
    }

    pub fn run_pregame(&mut self) -> Vec<Event> {
        let mut events = self.swap_cards_by_role(Role::Asshole, Role::President, 2);
        events.append(&mut self.swap_cards_by_role(Role::ViceAsshole, Role::VicePresident, 1));
        self.set_starting_player();
        events
    }

    pub fn get_player(&self, id: Uuid) -> Option<&Player> {
        self.table.iter().find(|p| p.state.id == id)
    }

    pub fn get_role(&self, role: Role) -> Option<&Player> {
        self.table.iter().find(|p| p.state.role == Some(role))
    }

    pub fn get_role_mut(&mut self, role: Role) -> Option<&mut Player> {
        self.table.iter_mut().find(|p| p.state.role == Some(role))
    }

    pub fn current_player(&self) -> &Player {
        self.table
            .front()
            .expect("Should always have a current player")
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        self.table
            .front_mut()
            .expect("Should always have a current player")
    }

    fn get_player_mut(&mut self, id: Uuid) -> Option<&mut Player> {
        self.table.iter_mut().find(|p| p.state.id == id)
    }

    fn last_played_player(&self) -> Option<&Player> {
        self.history
            .iter()
            .filter(|ev| matches!(ev.action, Action::PlayCards { .. }))
            .map(|ev| ev.player_id)
            .next_back()
            .and_then(|player_id| self.get_player(player_id))
    }

    fn starting_player_and_card(&self) -> (Uuid, Card) {
        let mut starter_id_and_card: Option<(Uuid, Card)> = None;
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
                .find(|player| player.state.current_hand.contains(&three_card))
            {
                starter_id_and_card = Some((starter.state.id, three_card));
                break;
            }
        }
        starter_id_and_card.expect("Someone must have one of: 3C, 3S, 3H, 3D, 4S")
    }

    fn set_starting_player(&mut self) -> Card {
        let (starter_id, card) = self.starting_player_and_card();
        let idx = self
            .table
            .iter()
            .position(|p| p.state.id == starter_id)
            .expect("Someone must have one of these cards");
        self.table.rotate_left(idx);
        card
    }

    fn swap_cards_by_role(
        &mut self,
        asshole_role: Role,
        president_role: Role,
        num_cards: usize,
    ) -> Vec<Event> {
        let public_info = self.public_info();

        // generate events
        let events = match (
            self.get_role(asshole_role).map(|s| s.state.clone()),
            self.get_role_mut(president_role),
        ) {
            (None, None) => {
                log::warn!("No players found for either role, so not swapping any cards: {asshole_role:?}, {president_role:?}");
                vec![]
            }
            (None, Some(_)) => {
                log::error!("Found player for {president_role:?} but not for {asshole_role:?}, so not swapping cards");
                vec![]
            }
            (Some(_), None) => {
                log::error!("Found player for {president_role:?} but not for {asshole_role:?}, so not swapping cards");
                vec![]
            }
            (Some(asshole), Some(president)) => {
                let mut events = Vec::with_capacity(4);

                let asshole_id = asshole.id;
                let president_id = president.state.id;
                events.extend(asshole.top_k_cards(num_cards).iter().map(|&card| Event {
                    player_id: asshole_id,
                    action: Action::SendCard {
                        to: president_id,
                        card,
                    },
                }));

                let mut sent_cards: HashSet<Card> = HashSet::new();
                for _ in 0..num_cards {
                    let available_actions: Vec<_> = president
                        .state
                        .current_hand
                        .iter()
                        .filter_map(|&card| {
                            if !sent_cards.contains(&card) {
                                Some(Action::SendCard {
                                    to: asshole_id,
                                    card,
                                })
                            } else {
                                None
                            }
                        })
                        .collect();

                    // president/VP should send bottom cards strategically
                    let action = president.strategy.select_action(
                        &president.state,
                        &public_info,
                        &available_actions,
                    );
                    if let Action::SendCard { card, .. } = action {
                        sent_cards.insert(card);
                    }
                    events.push(Event {
                        player_id: president_id,
                        action,
                    })
                }
                events
            }
        };

        // process events
        for event in &events {
            if let Event {
                player_id,
                action: Action::SendCard { to, card },
            } = event
            {
                let send_player = self
                    .get_player_mut(*player_id)
                    .expect("Card-send event recorded by unknown player");
                send_player.state.current_hand.remove_card(card);
                log::info!("{} did: {}", send_player.state.name, event.action);
                let rec_player = self
                    .get_player_mut(*to)
                    .expect("Tried to send a card to unknown player");
                rec_player.state.current_hand.push(*card);
            }
        }
        events
    }

    pub fn still_playing(&self) -> bool {
        self.table
            .iter()
            .filter(|player| !player.state.current_hand.is_empty())
            .count()
            >= 2
    }

    pub fn start_new_game(&mut self) {
        // TODO: should enable option to shuffle seating order between games. something like:
        // players.shuffle(&mut thread_rng());
        // let table = VecDeque::from(players);

        // scan history to assign new roles for next game
        let mut worst_to_first = Vec::with_capacity(self.table.len());

        // asshole may still have cards left
        for player in &self.table {
            if !player.state.current_hand.is_empty() {
                worst_to_first.push(player.state.id);
            }
        }

        for &event in self.history.iter().rev() {
            if matches!(event.action, Action::PlayCards { .. })
                && !worst_to_first.contains(&event.player_id)
            {
                worst_to_first.push(event.player_id);
            }
        }

        let results_str = worst_to_first
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, p_id)| {
                let player = self
                    .get_player(*p_id)
                    .expect("ID that played in last game should still exist");
                format!("{}. {}", idx + 1, player.state.name)
            })
            .join("\n");
        log::info!("Game over! Results:\n{results_str}");

        // NOTE: assumes all roles are being used

        // clear roles before assigning new roles
        for player in self.table.iter_mut() {
            player.state.role = None;
        }

        if let Some(&asshole_id) = worst_to_first.first() {
            let player = self
                .get_player_mut(asshole_id)
                .expect("ID that played in last game should still exist");
            player.state.role = Some(Role::Asshole);
        }
        if let Some(&vice_asshole_id) = worst_to_first.get(1) {
            let player = self
                .get_player_mut(vice_asshole_id)
                .expect("ID that played in last game should still exist");
            player.state.role = Some(Role::ViceAsshole);
        }
        if let Some(&vp_id) = worst_to_first.get(worst_to_first.len() - 2) {
            let player = self
                .get_player_mut(vp_id)
                .expect("ID that played in last game should still exist");
            player.state.role = Some(Role::VicePresident);
        }
        if let Some(&prez_id) = worst_to_first.last() {
            let player = self
                .get_player_mut(prez_id)
                .expect("ID that played in last game should still exist");
            player.state.role = Some(Role::President);
        }

        self.top_card = None;
        self.history.clear();

        let mut deck = Deck::new();
        deck.reset_shuffle();
        let hand_size = deck.count() / self.table.len();
        for player in self.table.iter_mut() {
            player.state.current_hand = deck.deal(hand_size).into_iter().map_into().collect();
        }

        log::info!("New game!");
    }
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
                    player.state.name,
                    player.state.current_hand.len(),
                    player
                        .state
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
