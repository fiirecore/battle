use core::ops::Deref;

use rhai::INT;

use pokedex::{moves::{MoveCategory, Move}, types::PokemonType};

use crate::moves::usage::MoveUsage;

#[derive(Clone, Copy)]
pub struct ScriptMove(*const Move, *const MoveUsage);

impl ScriptMove {
    pub fn new(m: &Move, u: &MoveUsage) -> Self {
        Self(m as _, u as _)
    }

    pub fn get_category(&mut self) -> MoveCategory {
        self.category
    }
    pub fn get_type(&mut self) -> PokemonType {
        self.pokemon_type
    }
    pub fn get_crit_rate(&mut self) -> INT {
        self.u().crit_rate as INT
    }

    fn m(&self) -> &Move {
        unsafe { &*self.0 }
    }

    fn u(&self) -> &MoveUsage {
        unsafe { &*self.1 }
    }

}

impl Deref for ScriptMove {
    type Target = Move;

    fn deref(&self) -> &Self::Target {
        self.m()
    }
}
