use core::ops::{Deref, DerefMut};
use hashbrown::HashSet;

use pokedex::{moves::MoveId, pokemon::owned::OwnedPokemon};

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

pub struct HostPokemon {
    pub p: BattlePokemon,
    pub learnable_moves: HashSet<MoveId>,
    pub known: bool,
}

impl HostPokemon {
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

impl From<OwnedPokemon> for HostPokemon {
    fn from(p: OwnedPokemon) -> Self {
        Self {
            p: BattlePokemon::from(p),
            learnable_moves: Default::default(),
            known: false,
        }
    }
}

impl PokemonView for HostPokemon {
    // fn id(&self) -> &PokemonId {
    //     &self.pokemon.id
    // }

    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}

impl Deref for HostPokemon {
    type Target = BattlePokemon;

    fn deref(&self) -> &Self::Target {
        &self.p
    }
}

impl DerefMut for HostPokemon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.p
    }
}

impl core::fmt::Display for BattlePokemon {
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
