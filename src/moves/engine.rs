use rand::Rng;
use std::error::Error;

use pokedex::moves::Move;

use crate::{
    moves::{target::TargetLocation, MoveResult},
    pokemon::battle::BattlePokemon,
};

#[cfg(feature = "rhai")]
pub mod default;

#[cfg(feature = "rhai")]
pub use default::DefaultMoveEngine;

pub trait MoveEngine {
    type Error: Error;

    fn execute<'d, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        used_move: &Move,
        user: &BattlePokemon<'d>,
        targets: Vec<(TargetLocation, &BattlePokemon<'d>)>,
    ) -> Result<Vec<(TargetLocation, MoveResult)>, Self::Error>;
}
