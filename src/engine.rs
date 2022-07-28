//! Move and item execution engine

use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash};
use std::error::Error;

use rand::Rng;

use pokedex::{
    ailment::LiveAilment,
    item::Item,
    moves::Move,
    pokemon::{owned::SavedPokemon, Health},
};

use crate::{
    data::BattleData,
    moves::{damage::DamageResult, MoveCancel},
    pokemon::{
        stat::{BattleStatType, Stage},
        Indexed, PokemonIdentifier,
    },
};

pub mod pokemon;
pub use pokemon::BattlePokemon;

pub trait BattleEngine {
    // type PokemonState;

    type MoveError: Error;
    
    type ItemError: Error;

    fn execute_move<
        ID: Clone + Hash + Eq + Debug + 'static,
        R: Rng + Clone + 'static,
        PLR: Players<ID>,
    >(
        &self,
        random: &mut R,
        used_move: &Move,
        user: Indexed<ID, &BattlePokemon>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::MoveError>;

    fn execute_item<ID: PartialEq, R: Rng, PLR: Players<ID>>(
        &self,
        battle: &BattleData,
        random: &mut R,
        item: &Item,
        user: &ID,
        target: PokemonIdentifier<ID>,
        players: &mut PLR,
    ) -> Result<Vec<ItemResult>, Self::ItemError>;

    fn update(&self);
}

pub trait Players<ID: PartialEq> {
    fn create_targets<R: Rng>(
        &self,
        user: &PokemonIdentifier<ID>,
        m: &Move,
        targeting: Option<PokemonIdentifier<ID>>,
        random: &mut R,
    ) -> Vec<PokemonIdentifier<ID>>;

    fn get(&self, id: &PokemonIdentifier<ID>) -> Option<&BattlePokemon>;

    fn get_mut(&mut self, id: &PokemonIdentifier<ID>) -> Option<&mut BattlePokemon>;

    fn take(&mut self, id: &PokemonIdentifier<ID>) -> Option<BattlePokemon>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Damage(DamageResult<Health>),
    Heal(i16),
    Ailment(Option<LiveAilment>),
    Stat(BattleStatType, Stage),
    Cancel(MoveCancel),
    Custom,
    Miss,
    Error,
}

pub enum ItemResult {
    Catch(SavedPokemon),
}
