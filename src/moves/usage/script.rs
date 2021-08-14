use rand::Rng;
use std::error::Error;

use crate::{
    moves::{Move, usage::MoveResult},
    pokemon::battle::BattlePokemon,
};

#[cfg(feature = "rhai")]
pub mod rhai;

pub trait MoveEngine {
    type Error: Error;

    fn execute<'a, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        used_move: &Move,
        user: &BattlePokemon<'a>,
        target: &BattlePokemon<'a>,
        is_user: bool,
    ) -> Result<Vec<MoveResult>, Self::Error>;
}
