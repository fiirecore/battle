//! Move execution engine

use core::hash::Hash;
use rand::Rng;
use std::error::Error;

use pokedex::{ailment::LiveAilment, moves::Move, pokemon::Health};

use crate::{
    moves::damage::DamageResult,
    pokemon::{
        stat::{BattleStatType, Stage},
        Indexed, PokemonIdentifier,
    },
};

pub mod prelude {
    pub use super::{BattlePokemon, MoveEngine, MoveResult, Players};

    #[cfg(feature = "default_engine")]
    pub use super::default::DefaultMoveEngine;
}

#[cfg(feature = "default_engine")]
pub mod default;

mod pokemon;
pub use pokemon::BattlePokemon;

pub trait MoveEngine {
    type Error: Error;

    fn execute<
        'd,
        ID: Clone + Hash + Eq + 'static,
        R: Rng + Clone + 'static,
        P: Players<'d, ID, R>,
    >(
        &self,
        random: &mut R,
        used_move: &Move,
        user: Indexed<ID, &BattlePokemon<'d>>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &P,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error>;
}

pub trait Players<'d, ID: PartialEq, R: Rng> {
    fn create_targets(
        &self,
        user: &PokemonIdentifier<ID>,
        m: &Move,
        targeting: Option<PokemonIdentifier<ID>>,
        random: &mut R,
    ) -> Vec<PokemonIdentifier<ID>>;

    fn get(&self, id: &PokemonIdentifier<ID>) -> Option<&BattlePokemon<'d>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Damage(DamageResult<Health>),
    Heal(i16),
    Ailment(LiveAilment),
    Stat(BattleStatType, Stage),
    Flinch,
    Miss,
    Error,
}
