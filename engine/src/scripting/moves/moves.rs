use rhai::INT;

use battle::{
    moves::{Accuracy, BattleMove, MoveCategory},
    pokedex::types::PokemonType,
};

#[derive(Clone, Copy)]
pub struct ScriptMove {
    pub category: MoveCategory,
    pub type_: PokemonType,
    pub accuracy: Option<Accuracy>,
    pub crit_rate: INT,
}

impl ScriptMove {
    pub fn new(m: &BattleMove) -> Self {
        Self {
            category: m.category,
            type_: m.pokemon_type,
            accuracy: m.accuracy,
            crit_rate: m.crit_rate as _,
        }
    }

    pub fn get_category(&mut self) -> MoveCategory {
        self.category
    }
    pub fn get_type(&mut self) -> PokemonType {
        self.type_
    }
    pub fn get_crit_rate(&mut self) -> INT {
        self.crit_rate
    }
}
