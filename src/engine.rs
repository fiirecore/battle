//! Move execution engine

use core::hash::Hash;
use rand::Rng;
use std::error::Error;

use pokedex::{ailment::LiveAilment, pokemon::Health};

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
        ID: Clone + Hash + Eq + 'static + core::fmt::Debug,
        R: Rng + Clone + 'static,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
        PLR: Players<ID, R, P, M, I>,
    >(
        &self,
        random: &mut R,
        used_move: &Move,
        user: Indexed<ID, &BattlePokemon<P, M, I>>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error>;
}

use core::ops::Deref;
use pokedex::{item::Item, moves::Move, pokemon::Pokemon};

pub trait Players<
    ID: PartialEq,
    R: Rng,
    P: Deref<Target = Pokemon>,
    M: Deref<Target = Move>,
    I: Deref<Target = Item>,
>
{
    fn create_targets(
        &self,
        user: &PokemonIdentifier<ID>,
        m: &Move,
        targeting: Option<PokemonIdentifier<ID>>,
        random: &mut R,
    ) -> Vec<PokemonIdentifier<ID>>;

    fn get(&self, id: &PokemonIdentifier<ID>) -> Option<&BattlePokemon<P, M, I>>;
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
