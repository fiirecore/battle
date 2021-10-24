use core::ops::{Deref, DerefMut};
use hashbrown::HashSet;

use pokedex::{moves::MoveId, pokemon::owned::OwnedPokemon, Uninitializable};

use crate::{engine::BattlePokemon, moves::BattleMove, party::{Active, ActivePokemon}, pokemon::{
        remote::{RemotePokemon, UnknownPokemon},
        PokemonView,
    }};

#[derive(Debug, Clone, Copy)]
pub struct ActiveBattlePokemon<ID> {
    pub index: usize,
    pub queued_move: Option<BattleMove<ID>>,
}

impl<ID> ActiveBattlePokemon<ID> {
    pub fn as_usize<const AS: usize>(this: &Active<Self, AS>) -> Active<usize, AS> {
        let mut active = [None; AS];

        for (i, a) in this.iter().enumerate() {
            active[i] = a.as_ref().map(ActivePokemon::index);
        }

        return active;
    }

    pub fn queued(&self) -> bool {
        self.queued_move.is_some()
    }
}

pub struct HostPokemon<'d> {
    pub p: BattlePokemon<'d>,
    pub learnable_moves: HashSet<MoveId>,
    pub known: bool,
}

impl<'d> HostPokemon<'d> {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{}, Queued move: {}",
            self.index,
            self.queued_move.is_some()
        )
    }
}

impl<'d> From<OwnedPokemon<'d>> for HostPokemon<'d> {
    fn from(p: OwnedPokemon<'d>) -> Self {
        Self {
            p: BattlePokemon::from(p),
            learnable_moves: Default::default(),
            known: false,
        }
    }
}

impl<'d> PokemonView for HostPokemon<'d> {
    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}

impl<'d> Deref for HostPokemon<'d> {
    type Target = BattlePokemon<'d>;

    fn deref(&self) -> &Self::Target {
        &self.p
    }
}

impl<'d> DerefMut for HostPokemon<'d> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.p
    }
}

impl<'d> core::fmt::Display for BattlePokemon<'d> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
