use hashbrown::HashMap;
use rand::Rng;

use pokedex::{
    moves::{MoveCategory, MoveId},
    types::PokemonType,
};

use crate::{
    moves::{Move, usage::{script::MoveEngine, DamageResult, MoveResult}},
    pokemon::battle::BattlePokemon,
};

use rhai::{exported_module, plugin::*, Array, Dynamic, Engine, ParseError, Scope, AST, INT};

mod damage;
mod error;
mod moves;
mod pokemon;
mod random;

pub use damage::*;
pub use error::*;
pub use moves::*;
pub use pokemon::*;
pub use random::*;
pub use result::*;

pub type Scripts = HashMap<(MoveId, bool), AST>;

pub struct RhaiMoveEngine {
    scripts: Scripts,
    engine: Engine,
}

impl RhaiMoveEngine {
    pub fn new<R: Rng + Clone + 'static>() -> Self {
        let mut engine = Engine::new_raw();

        engine
            .register_type_with_name::<ScriptRandom<R>>("Random")
            .register_type_with_name::<DamageResult<INT>>("Damage")
            .register_set("damage", ScriptDamage::set_damage)
            .register_get("damage", ScriptDamage::get_damage)
            .register_get("effective", ScriptDamage::effective)
            .register_type_with_name::<ScriptPokemon<R>>("Pokemon")
            .register_fn("damage", ScriptPokemon::<R>::get_damage)
            .register_get("hp", ScriptPokemon::<R>::hp) 
            .register_type::<ScriptMove>()
            .register_get("category", ScriptMove::get_category)
            .register_get("type", ScriptMove::get_type)
            .register_get("crit_rate", ScriptMove::get_crit_rate)
            .register_type_with_name::<MoveCategory>("Category")
            .register_type_with_name::<PokemonType>("Type")
            .register_type_with_name::<MoveResult>("MoveResult")
            .register_static_module("MoveResult", exported_module!(result).into());

        Self {
            engine,
            scripts: Default::default(),
        }
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.scripts.reserve(capacity)
    }

    pub fn insert(&mut self, id: MoveId, is_user: bool, script: &str) -> Result<(), ParseError> {
        self.scripts
            .insert((id, is_user), self.engine.compile(script)?);
        Ok(())
    }
}

impl MoveEngine for RhaiMoveEngine {
    type Error = RhaiMoveError;

    fn execute<'a, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        used_move: &Move,
        user: &BattlePokemon<'a>,
        target: &BattlePokemon<'a>,
        is_user: bool,
    ) -> Result<Vec<MoveResult>, Self::Error> {
        match self.scripts.get(&(used_move.id, is_user)) {
            Some(script) => {
                let mut scope = Scope::new();
                scope.push("random", ScriptRandom::new(random));
                scope.push("move", ScriptMove::new(used_move));
                scope.push("user", ScriptPokemon::<R>::new(user));
                scope.push("target", ScriptPokemon::<R>::new(target));

                Ok(self
                    .engine
                    .eval_ast_with_scope::<Array>(&mut scope, script)?
                    .into_iter()
                    .flat_map(Dynamic::try_cast::<MoveResult>)
                    .collect())
            }
            None => Err(RhaiMoveError::Missing),
        }
    }
}

#[allow(non_snake_case, non_upper_case_globals)]
#[export_module]
mod result {
    use rhai::INT;

    use crate::moves::usage::MoveResult;

    use super::ScriptDamage;

    pub fn Damage(damage: ScriptDamage) -> MoveResult {
        MoveResult::Damage(damage.into())
    }
    // pub const fn Status(effect: StatusEffect) -> MoveResult { MoveResult::Status(effect) }
    pub fn Drain(damage: ScriptDamage, heal: INT) -> MoveResult {
        MoveResult::Drain(damage.into(), heal as _)
    }
}
