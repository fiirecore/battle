//! Move and item execution engine

use alloc::vec::Vec;
use hashbrown::HashMap;
use core::{fmt::Debug, hash::Hash};
use std::error::Error;

use rand::Rng;

use pokedex::{ailment::LiveAilment, item::ItemId, moves::MoveId, pokemon::Health};

use crate::{
    data::BattleData,
    host::BattlePlayer,
    moves::{BattleMove, DamageResult, MoveCancelId, RemovePokemonId},
    pokemon::{
        stat::{BattleStatType, Stage},
        Indexed, TeamIndex, ActivePosition,
    },
    select::{ClientMoveAction, BattleSelection, SelectMessage},
};

pub mod pokemon;
pub use pokemon::BattlePokemon;

pub trait BattleEngine<ID: Clone + Hash + Eq + 'static, T>: Send + Sync + 'static {
    // type PokemonState;

    type ExecutionError: Error;

    type Data: Default;

    /// subtract pp on successful move use, todo subtract item
    fn select(
        &self,
        data: &mut Self::Data,
        active: ActivePosition,
        selection: &BattleSelection<ID>,
        player: &mut BattlePlayer<ID, T>,
    ) -> SelectMessage;

    /// execute a single move or item
    fn execute(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        action: ExecuteAction<ID>,
        players: PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, ClientMoveAction>>, Self::ExecutionError>;

    /// run the actions after the moves finish
    fn post(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        players: PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, ClientMoveAction>>, Self::ExecutionError>;

    fn get_move(&self, id: &MoveId) -> Option<&BattleMove>;
}

pub struct PlayerQuery<'a, ID, T>(pub(crate) &'a mut Vec<BattlePlayer<ID, T>>);

impl<'a, ID, T> PlayerQuery<'a, ID, T> {

    pub fn iter(&'a self) -> impl DoubleEndedIterator<Item = &'a BattlePlayer<ID, T>> + 'a {
        self.0.iter().filter(|p| p.removed.is_none())
    }

    pub fn iter_mut(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut BattlePlayer<ID, T>> + 'a {
        self.0.iter_mut().filter(|p| p.removed.is_none())
    }

}

pub enum ExecuteAction<'a, ID> {
    Move(
        &'a MoveId,
        &'a TeamIndex<ID>,
        Option<&'a TeamIndex<ID>>,
    ),
    Item(&'a ItemId, &'a ID, TeamIndex<ID>),
}

pub struct ExecuteResult<ID> {
    pub global: Vec<Vec<Indexed<ID, ClientMoveAction>>>,
    pub unique: HashMap<TeamIndex<ID>, Vec<Indexed<ID, ClientMoveAction>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResult {
    Damage(DamageResult<Health>),
    Heal(i16),
    Ailment(Option<LiveAilment>),
    Stat(BattleStatType, Stage),
    /// partial reveal = false, full reveal = true
    Reveal(bool),
    Cancel(MoveCancelId),
    Remove(RemovePokemonId),
    Fail,
    Miss,
}
