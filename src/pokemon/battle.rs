use core::ops::{Deref, DerefMut};
use hashbrown::HashSet;

use pokedex::{
    moves::{Move, MoveId},
    pokemon::{
        owned::OwnedPokemon,
        stat::{BaseStat, StatType},
    },
};

mod unknown;
pub use unknown::*;

pub mod stat;

mod moves;

use crate::pokemon::battle::stat::{BattleStatType, StatStages};

#[derive(Debug)]
pub struct BattlePokemon<'d> {
    pub instance: OwnedPokemon<'d>,
    pub learnable_moves: HashSet<MoveId>,
    // pub persistent: Option<PersistentMove>,
    pub caught: bool,
    pub known: bool,
    pub flinch: bool,
    pub requestable: bool,
    pub stages: StatStages,
}

impl<'d> BattlePokemon<'d> {
    pub fn know(&mut self) -> Option<InitUnknownPokemon<'d>> {
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

    // To - do: factor in accuracy
    pub fn throw_move<R: rand::Rng>(&self, random: &mut R, m: &Move) -> bool {
        m.accuracy
            .map(|accuracy| random.gen_range(0..100) < accuracy)
            .unwrap_or(true)
    }

    pub fn stat(&self, stat: StatType) -> BaseStat {
        StatStages::mult(
            self.instance.stat(stat),
            self.stages.get(BattleStatType::Basic(stat)),
        )
    }
}

impl<'d> From<OwnedPokemon<'d>> for BattlePokemon<'d> {
    fn from(instance: OwnedPokemon<'d>) -> Self {
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
    type Target = OwnedPokemon<'d>;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl<'d> DerefMut for BattlePokemon<'d> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}
