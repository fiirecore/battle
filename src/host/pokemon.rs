use core::ops::{Deref, DerefMut};
use hashbrown::HashSet;

use pokedex::{
    item::Item,
    moves::{Move, MoveId},
    pokemon::{owned::OwnedPokemon, Pokemon},
    Uninitializable,
};

use crate::{
    engine::BattlePokemon,
    moves::BattleMove,
    party::{Active, ActivePokemon},
    pokemon::{
        remote::{RemotePokemon, UnknownPokemon},
        PokemonView,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ActiveBattlePokemon<ID> {
    pub index: usize,
    pub queued_move: Option<BattleMove<ID>>,
}

impl<ID> ActiveBattlePokemon<ID> {
    pub fn as_usize(this: &[Option<Self>]) -> Active<usize> {
        this.iter()
            .map(|o| o.as_ref().map(ActivePokemon::index))
            .collect()
    }

    pub fn queued(&self) -> bool {
        self.queued_move.is_some()
    }
}

pub struct HostPokemon<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>>
{
    pub p: BattlePokemon<P, M, I>,
    pub learnable_moves: HashSet<MoveId>,
    pub known: bool,
}

impl<P: Deref<Target = Pokemon> + Clone, M: Deref<Target = Move>, I: Deref<Target = Item>> HostPokemon<P, M, I> {
    pub fn know(&mut self) -> Option<RemotePokemon> {
        (!self.known).then(|| {
            self.known = true;
            UnknownPokemon::new(&self.p).uninit()
        })
    }
}

impl<ID> ActivePokemon for ActiveBattlePokemon<ID> {
    fn index(&self) -> usize {
        self.index
    }
}

impl<ID> From<usize> for ActiveBattlePokemon<ID> {
    fn from(index: usize) -> Self {
        Self {
            index,
            queued_move: None,
        }
    }
}

impl<ID> core::fmt::Display for ActiveBattlePokemon<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "#{}, Queued move: {}",
            self.index,
            self.queued_move.is_some()
        )
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>> From<OwnedPokemon<P, M, I>> for HostPokemon<P, M, I> {
    fn from(p: OwnedPokemon<P, M, I>) -> Self {
        Self {
            p: BattlePokemon::from(p),
            learnable_moves: Default::default(),
            known: false,
        }
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>> PokemonView for HostPokemon<P, M, I> {
    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>> Deref for HostPokemon<P, M, I> {
    type Target = BattlePokemon<P, M, I>;

    fn deref(&self) -> &Self::Target {
        &self.p
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>> DerefMut for HostPokemon<P, M, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.p
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>> core::fmt::Display for BattlePokemon<P, M, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\"{}\": {}, {}/{} HP",
            self.name(),
            self.level,
            self.hp(),
            self.max_hp()
        )
    }
}
