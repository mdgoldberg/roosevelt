use std::collections::VecDeque;

use deckofcards::{Deck, Rank, Suit};
use itertools::Itertools;
use log;
use rand::seq::SliceRandom;
use rand::thread_rng;
use uuid::Uuid;

use crate::types::{Action, Card, CardPlay, Event, GameState, Player, Role};

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
                let cards: Vec<_> = deck.deal(hand_size).into_iter().map_into().collect();
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

    pub fn run_game(&mut self) {
        assert_eq!(self.history.len(), 0);
        self.run_pregame();
        while self.still_playing() {
            log::info!("{self}");
            // sleep(Duration::from_secs(1));
            let available_actions = self.permitted_actions();
            let selected_action = self
                .current_player()
                .select_action(&self, &available_actions);
            self.perform_ingame_action(&selected_action);
        }
        self.start_new_game();
    }

    pub fn permitted_actions(&self) -> Vec<Action> {
        let current_player = self.current_player();
        let hand = &current_player.current_hand;
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
        if let Some(_) = self.top_card {
            actions.push(Action::Pass);
        }
        // first card play must contain starting card
        let is_first_cardplay = self
            .history
            .iter()
            .all(|ev| !matches!(ev.action, Action::PlayCards { .. }));
        if is_first_cardplay {
            let (_, starting_card) = self.starting_player_and_card();
            actions.retain(|action| match action {
                Action::PlayCards { card_play } => {
                    card_play.to_vec().iter().any(|&card| card == starting_card)
                }
                _ => false,
            });
        }

        // log::info!("Actions for {}: {actions}", current_player.name);
        actions
    }

    pub fn perform_ingame_action(&mut self, action: &Action) {
        let player = self.current_player_mut();
        let player_id = player.id;
        match action {
            Action::SendCard { .. } => {
                panic!("Attempted to send a card in the middle of the game!");
            }
            Action::Pass => {}
            Action::PlayCards { card_play } => {
                for card in &card_play.to_vec() {
                    let removed = player.current_hand.remove_card(card);
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
        log::info!("{} did: {action}", self.current_player());
        // record event in history
        let event = Event {
            player_id,
            action: *action,
        };
        self.history.push(event);

        // also handles clearing the deck if necessary
        self.next_players_turn();
        while self.current_player().current_hand.is_empty() {
            self.next_players_turn();
        }
    }

    fn next_players_turn(&mut self) {
        self.table.rotate_left(1);

        // clear the deck if necessary
        if self.last_played_player() == Some(self.current_player()) {
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

    fn current_player_mut(&mut self) -> &mut Player {
        self.table
            .front_mut()
            .expect("Should always have a current player")
    }

    fn last_played_player(&self) -> Option<&Player> {
        self.history
            .iter()
            .filter(|ev| matches!(ev.action, Action::PlayCards { .. }))
            .map(|ev| ev.player_id)
            .last()
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
                .find(|player| player.current_hand.contains(&three_card))
            {
                starter_id_and_card = Some((starter.id, three_card));
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
            .position(|p| p.id == starter_id)
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
        // generate events
        let events = match (self.get_role(asshole_role), self.get_role(president_role)) {
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
                let president_id = president.id;
                events.extend(asshole.top_k_cards(num_cards).iter().map(|&card| Event {
                    player_id: asshole_id,
                    action: Action::SendCard {
                        to: president_id,
                        card,
                    },
                }));

                events.extend((0..num_cards).map(|_| {
                    let available_actions: Vec<_> = president
                        .current_hand
                        .iter()
                        .map(|&card| Action::SendCard {
                            to: asshole_id,
                            card,
                        })
                        .collect();
                    // president/VP should send bottom cards strategically
                    let action = president.select_action(self, &available_actions);
                    Event {
                        player_id: president_id,
                        action,
                    }
                }));

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
                send_player.current_hand.remove_card(card);
                let rec_player = self
                    .get_player_mut(*to)
                    .expect("Tried to send a card to unknown player");
                rec_player.current_hand.push(*card);
            }
        }
        events
    }

    pub fn still_playing(&self) -> bool {
        self.table
            .iter()
            .filter(|player| !player.current_hand.is_empty())
            .count()
            >= 2
    }

    pub fn start_new_game(&mut self) {
        // TODO: should enable option to shuffle seating order between games

        // TODO: scan history to assign new roles for next game
        todo!()
    }
}

impl Player {
    pub fn new(name: &str, dealt_hand: Vec<Card>, role: Option<Role>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            role,
            current_hand: dealt_hand,
        }
    }

    pub fn select_action<'a>(
        &self,
        game_state: &GameState,
        available_actions: &[Action],
    ) -> Action {
        // TODO
        let card_play = available_actions
            .iter()
            .filter_map(|action| {
                if let Action::PlayCards { card_play } = action {
                    Some(card_play)
                } else {
                    None
                }
            })
            .min_by_key(|cp| (cp.size(), cp.value()))
            .map(|&card_play| Action::PlayCards { card_play });

        card_play.unwrap_or(available_actions[0])
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

trait Hand {
    fn remove_card(&mut self, card: &Card) -> bool;
    fn singles(&self) -> Vec<CardPlay>;
    fn pairs(&self) -> Vec<CardPlay>;
    fn triples(&self) -> Vec<CardPlay>;
    fn quads(&self) -> Vec<CardPlay>;
}

fn _card_plays_for_size(hand: &[Card], card_play_size: usize) -> Vec<CardPlay> {
    hand.iter()
        .combinations(card_play_size)
        .filter_map(|cards| {
            let rank = cards.get(0)?.rank();
            if cards.iter().all(|c| c.rank() == rank) {
                Some(CardPlay::from_cards(&cards))
            } else {
                None
            }
        })
        .collect()
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

    fn singles(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 1)
    }

    fn pairs(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 2)
    }

    fn triples(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 3)
    }

    fn quads(&self) -> Vec<CardPlay> {
        _card_plays_for_size(self, 4)
    }
}

impl CardPlay {
    pub fn from_cards(cards: &[&Card]) -> CardPlay {
        match cards {
            &[card] => CardPlay::Single(*card),
            &[card1, card2] => CardPlay::Pair(*card1, *card2),
            &[card1, card2, card3] => CardPlay::Triple(*card1, *card2, *card3),
            &[card1, card2, card3, card4] => CardPlay::Quad(*card1, *card2, *card3, *card4),
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
