use rand::Rng;
use std::error::Error;

use pokedex::moves::Move;

use crate::{
    moves::{usage::MoveResult},
    pokemon::battle::BattlePokemon,
};

#[cfg(feature = "rhai")]
mod default;

#[cfg(feature = "rhai")]
pub use default::*;

pub trait MoveEngine {
    type Error: Error;

    fn execute<'a, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        used_move: &Move,
        user: &BattlePokemon<'a>,
        target: &BattlePokemon<'a>,
    ) -> Result<Vec<MoveResult>, Self::Error>;
}
