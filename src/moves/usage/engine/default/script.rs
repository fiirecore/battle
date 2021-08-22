use core::marker::PhantomData;
use hashbrown::HashMap;
use rand::Rng;
use rhai::{exported_module, plugin::*, Array, Dynamic, Engine, Scope, AST, INT};

use pokedex::{
    moves::{Move, MoveCategory},
    types::PokemonType,
};

use crate::{
    moves::usage::{engine::MoveScriptEngine, DamageResult, MoveResult, MoveScriptId, MoveUsage},
    pokemon::battle::BattlePokemon,
};

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

pub type Scripts = HashMap<MoveScriptId, AST>;

pub struct RhaiMoveScriptEngine<R: Rng + Clone + 'static> {
    pub scripts: Scripts,
    pub engine: Engine,
    _marker: PhantomData<R>,
}

impl<R: Rng + Clone + 'static> RhaiMoveScriptEngine<R> {
    pub fn new() -> Self {
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
            scripts: Default::default(),
            engine,
            _marker: PhantomData,
        }
    }
}

impl<RNG: Rng + Clone + 'static> MoveScriptEngine for RhaiMoveScriptEngine<RNG> {
    type Error = RhaiMoveError;

    fn execute<'a, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        used_move: &Move,
        usage: &MoveUsage,
        id: &MoveScriptId,
        user: &BattlePokemon<'a>,
        target: &BattlePokemon<'a>,
    ) -> Result<Vec<MoveResult>, Self::Error> {
        match self.scripts.get(&id) {
            Some(script) => {
                let mut scope = Scope::new();
                scope.push("random", ScriptRandom::new(random));
                scope.push("move", ScriptMove::new(used_move, usage));
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
