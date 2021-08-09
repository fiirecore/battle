use core::ops::{Deref, DerefMut};
use hashbrown::HashSet;

use pokedex::{
    moves::MoveId,
    pokemon::{
        stat::{BaseStat, StatType},
        InitPokemon, PokemonRef,
    },
};

mod stat;
pub use stat::*;

use crate::pokemon::UnknownPokemon;

#[derive(Debug, Clone)]
pub struct BattlePokemon<'d> {
    pub instance: InitPokemon<'d>,
    pub learnable_moves: HashSet<MoveId>,
    // pub persistent: Option<PersistentMove>,
    pub caught: bool,
    pub known: bool,
    pub flinch: bool,
    pub requestable: bool,
    pub stages: StatStages,
}

impl<'d> BattlePokemon<'d> {
    pub fn know(&mut self) -> Option<UnknownPokemon<PokemonRef<'d>>> {
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

    pub fn stat(&self, stat: StatType) -> BaseStat {
        StatStages::mult(self.instance.stat(stat), self.stages.get(stat))
    }

}

impl<'d> From<InitPokemon<'d>> for BattlePokemon<'d> {
    fn from(instance: InitPokemon<'d>) -> Self {
        Self {
            instance,
            learnable_moves: Default::default(),
            caught: false,
            known: false,
            flinch: false,
            requestable: false,
            stages: Default::default(),
        }
    }
}

impl<'d> Deref for BattlePokemon<'d> {
    type Target = InitPokemon<'d>;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl<'d> DerefMut for BattlePokemon<'d> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}
