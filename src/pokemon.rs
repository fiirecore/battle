use serde::{Deserialize, Serialize};
use core::{hash::Hash, fmt::Display, ops::{Deref, DerefMut}};

use pokedex::{
    pokemon::PokemonInstance,
    moves::MoveId,
};

mod active;
pub use active::*;

mod unknown;
pub use unknown::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PokemonIndex<ID> {
    pub team: ID,
    pub index: usize,
}

impl<ID: Sized + Display> Display for PokemonIndex<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} #{}", self.team, self.index)
    }
}

#[derive(Debug, Clone)]
pub struct BattlePokemon {
    pub instance: PokemonInstance,
    pub learnable_moves: Vec<MoveId>,
    // pub persistent: Option<PersistentMove>,
    pub caught: bool,
    pub known: bool,
    pub flinch: bool,
    pub requestable: bool,
}

impl BattlePokemon {
    pub fn know(&mut self) -> Option<UnknownPokemon> {
        (!self.known).then(|| {
            self.known = true;
            UnknownPokemon::new(&self.instance)
        })
    }

    pub fn try_flinch(&mut self) -> bool {
        if self.flinch {
            self.flinch = false;
            true
        } else {
            false
        }
    }
}

impl From<PokemonInstance> for BattlePokemon {
    fn from(instance: PokemonInstance) -> Self {
        Self {
            instance,
            learnable_moves: Vec::new(),
            caught: false,
            known: false,
            flinch: false,
            requestable: false,
        }
    }
}

impl Deref for BattlePokemon {
    type Target = PokemonInstance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl DerefMut for BattlePokemon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}