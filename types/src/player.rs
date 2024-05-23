use std::fmt::Debug;

use crate::{game_state::PublicInfo, Action, PlayerState};

pub trait Strategy: Debug {
    fn select_action(
        &mut self,
        private_info: &PlayerState,
        public_info: &PublicInfo,
        available_actions: &[Action],
    ) -> Action;
}

#[derive(Debug)]
pub struct Player {
    pub state: PlayerState,
    pub strategy: Box<dyn Strategy>,
}
