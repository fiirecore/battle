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

use super::{collections::BattleMap, player::BattlePlayer, pokemon::BattlePokemon};

#[cfg(feature = "default_engine")]
pub mod default;

pub trait MoveEngine {
    type Error: Error;

    fn execute<'d, ID: Clone + Hash + Eq + 'static, R: Rng + Clone + 'static, const AS: usize>(
        &self,
        random: &mut R,
        used_move: &Move,
        user: Indexed<ID, &BattlePokemon<'d>>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &BattleMap<ID, BattlePlayer<'d, ID, AS>>,
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
