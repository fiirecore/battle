use rhai::INT;

use pokedex::{
    moves::{Move, MoveCategory},
    types::PokemonType,
};

#[derive(Clone, Copy)]
pub struct ScriptMove(*const Move);

impl ScriptMove {
    pub fn new(m: &Move) -> Self {
        Self(m as _)
    }

    pub fn get_category(&mut self) -> MoveCategory {
        self.m().category
    }
    pub fn get_type(&mut self) -> PokemonType {
        self.m().pokemon_type
    }
    pub fn get_crit_rate(&mut self) -> INT {
        self.m().crit_rate as INT
    }

    pub fn m(&self) -> &Move {
        unsafe { &*self.0 }
    }
}