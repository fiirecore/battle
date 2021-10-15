use core::hash::Hash;
use rand::Rng;
use std::error::Error;

use pokedex::{ailment::LiveAilment, moves::Move, pokemon::Health};

use crate::{
    moves::damage::DamageResult,
    pokemon::{
        stat::{BattleStatType, Stage},
        PokemonIdentifier,
    },
    BattleEndpoint, Indexed,
};

use super::{player::BattlePlayer, pokemon::BattlePokemon, collections::BattleMap};

#[cfg(feature = "default_engine")]
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
        &self,
        random: &mut R,
        used_move: &Move,
        user: Indexed<ID, &BattlePokemon<'d>>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &BattleMap<ID, BattlePlayer<'d, ID, E, AS>>,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error>;
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
