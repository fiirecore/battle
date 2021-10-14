use core::hash::Hash;
use rand::Rng;
use std::error::Error;

use pokedex::{ailment::LiveAilment, moves::Move, pokemon::Health};

use crate::{
    moves::damage::DamageResult,
    player::BattlePlayer,
    pokemon::{
        battle::{
            stat::{BattleStatType, Stage},
            BattlePokemon,
        },
        PokemonIndex,
    },
    prelude::BattleMap,
    BattleEndpoint,
};

pub use hashbrown::HashMap;

#[cfg(feature = "scripting")]
pub mod default;

pub trait MoveEngine {
    type Error: Error;

    fn execute<
        'd,
        ID: Clone + Hash + Eq + 'static,
        R: Rng + Clone + 'static,
        E: BattleEndpoint<ID, AS>,
        const AS: usize,
    >(
        &mut self,
        random: &mut R,
        used_move: &Move,
        user: (PokemonIndex<ID>, &BattlePokemon<'d>),
        targets: Option<PokemonIndex<ID>>,
        players: &BattleMap<ID, BattlePlayer<'d, ID, E, AS>>,
    ) -> Result<HashMap<PokemonIndex<ID>, Vec<MoveResult>>, Self::Error>;
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
