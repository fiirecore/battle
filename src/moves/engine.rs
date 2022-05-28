//! Move execution engine

use core::hash::Hash;
use rand::Rng;
use std::error::Error;

use pokedex::{ailment::LiveAilment, pokemon::Health};

use crate::{
    engine::{BattlePokemon, Players},
    moves::{damage::DamageResult, MoveCancel},
    pokemon::{
        stat::{BattleStatType, Stage},
        Indexed, PokemonIdentifier,
    },
};

pub trait MoveEngine {
    type Error: Error;

    fn execute<
        ID: Clone + Hash + Eq + 'static + core::fmt::Debug,
        R: Rng + Clone + 'static,
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
        PLR: Players<ID, P, M, I>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Damage(DamageResult<Health>),
    Heal(i16),
    Ailment(LiveAilment),
    Stat(BattleStatType, Stage),
    Cancel(MoveCancel),
    Miss,
    Error,
}