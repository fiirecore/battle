//! Move and item execution engine

use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash};
use hashbrown::HashMap;
use std::error::Error;

use rand::Rng;

use pokedex::{ailment::LiveAilment, item::ItemId, moves::MoveId, pokemon::Health};

use crate::{
    data::BattleData,
    moves::{BattleMove, DamageResult, MoveCancelId, RemovePokemonId},
    pokemon::{
        stat::{BattleStatType, Stage},
        ActivePosition, BattlePokemon, Indexed, TeamIndex,
    },
    select::{BattleSelection, PublicAction, SelectMessage},
};

mod active;
mod player;

pub use player::BattlePlayer;
pub(crate) use {active::ActiveBattlePokemon, player::PlayerEndpoint};

pub trait BattleEngine<ID: Clone + Hash + Eq + 'static, T>: Send + Sync + 'static {
    // type PokemonState;

    type ExecutionError: Error;

    type Data: Default;

    /// subtract pp on successful move use, todo subtract item
    /// DOES NOT RUN FOR SWITCH
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
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError>;

    /// run the actions after the moves finish
    fn post(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError>;

    fn reset(&self, data: &mut Self::Data);

    fn get_move(&self, id: &MoveId) -> Option<&BattleMove>;
}


pub struct PlayerQuery<ID, T>(Vec<BattlePlayer<ID, T>>);

impl<ID: PartialEq, T> PlayerQuery<ID, T> {

    pub fn new(inner: Vec<BattlePlayer<ID, T>>) -> Self {
        Self(inner)
    }

    pub(crate) fn unfiltered_iter(&self) -> core::slice::Iter<'_, BattlePlayer<ID, T>> {
        self.0.iter()
    }

    pub(crate) fn unfiltered_iter_mut(&mut self) -> core::slice::IterMut<'_, BattlePlayer<ID, T>> {
        self.0.iter_mut()
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear()
    }

    pub(crate) fn extend(&mut self, iter: impl IntoIterator<Item = BattlePlayer<ID, T>>) {
        self.0.extend(iter)
    }

    pub(crate) fn get_index(&self, index: usize) -> Option<&BattlePlayer<ID, T>> {
        self.0.get(index)
    }

    pub(crate) fn get_index_mut(&mut self, index: usize) -> Option<&mut BattlePlayer<ID, T>> {
        self.0.get_mut(index)
    }

    fn query_filter(p: &&BattlePlayer<ID, T>) -> bool {
        p.removed.is_none() && p.is_ready()
    }

    fn query_filter_mut(p: &&mut BattlePlayer<ID, T>) -> bool {
        Self::query_filter(&&**p)
    }

    pub fn iter(
        &self,
    ) -> core::iter::Filter<
        core::slice::Iter<'_, BattlePlayer<ID, T>>,
        impl FnMut(&&BattlePlayer<ID, T>) -> bool,
    > {
        self.0.iter().filter(Self::query_filter)
    }

    pub fn iter_mut(
        &mut self,
    ) -> core::iter::Filter<
        core::slice::IterMut<'_, BattlePlayer<ID, T>>,
        impl FnMut(&&mut BattlePlayer<ID, T>) -> bool,
    > {
        self.0.iter_mut().filter(Self::query_filter_mut)
    }

    pub fn get(&self, id: &ID) -> Option<&BattlePlayer<ID, T>> {
        self.iter().find(|p| p.id() == id)
    }

    pub fn get_mut(&mut self, id: &ID) -> Option<&mut BattlePlayer<ID, T>> {
        self.iter_mut().find(|p| p.id() == id)
    }
}

pub enum ExecuteAction<'a, ID> {
    Move(&'a MoveId, &'a TeamIndex<ID>, Option<&'a TeamIndex<ID>>),
    Item(&'a ItemId, &'a ID, TeamIndex<ID>),
}

pub struct ExecuteResult<ID> {
    pub global: Vec<Vec<Indexed<ID, PublicAction>>>,
    pub unique: HashMap<TeamIndex<ID>, Vec<Indexed<ID, PublicAction>>>,
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

impl PublicAction {
    pub fn new(pokemon: &BattlePokemon, result: ActionResult) -> Self {
        match result {
            ActionResult::Damage(_) => todo!(),
            ActionResult::Heal(_) => todo!(),
            ActionResult::Ailment(_) => todo!(),
            ActionResult::Stat(_, _) => todo!(),
            ActionResult::Reveal(_) => todo!(),
            ActionResult::Cancel(_) => todo!(),
            ActionResult::Remove(_) => todo!(),
            ActionResult::Fail => todo!(),
            ActionResult::Miss => todo!(),
        }
    }
}
