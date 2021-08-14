use core::ops::Deref;

use rhai::INT;

use pokedex::{moves::MoveCategory, types::PokemonType};

use crate::moves::Move;

#[derive(Clone, Copy)]
pub struct ScriptMove(*const Move);

impl ScriptMove {
    pub fn new(m: &Move) -> Self {
        Self(m as *const _)
    }

    pub fn get_category(&mut self) -> MoveCategory {
        self.category
    }
    pub fn get_type(&mut self) -> PokemonType {
        self.pokemon_type
    }
    pub fn get_crit_rate(&mut self) -> INT {
        self.usage.crit_rate as INT
    }
}

impl Deref for ScriptMove {
    type Target = Move;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
