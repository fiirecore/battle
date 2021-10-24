use core::hash::Hash;
use hashbrown::HashMap;
use rand::Rng;

use rhai::{
    packages::{BasicArrayPackage, Package},
    Array, Dynamic, Engine, Scope, INT,
};

use pokedex::{
    moves::{Move, MoveCategory, MoveId},
    types::PokemonType,
};

mod damage;
mod moves;
mod pokemon;
mod random;
mod result;

use damage::*;
use moves::*;
use pokemon::*;
use random::*;
use result::*;

use crate::{
    engine::{BattlePokemon, MoveResult, Players},
    moves::damage::DamageResult,
    pokemon::{Indexed, PokemonIdentifier},
};

use super::DefaultMoveError;

pub type Scripts = HashMap<MoveId, String>;

pub struct DefaultScriptingEngine {
    pub scripts: Scripts,
    pub engine: Engine,
}

impl DefaultScriptingEngine {
    pub fn new<'d, ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        let mut engine = Engine::new_raw();

        engine
            .register_global_module(BasicArrayPackage::new().as_shared_module())
            .register_type_with_name::<ScriptRandom<R>>("Random")
            .register_type_with_name::<DamageResult<INT>>("Damage")
            .register_set("damage", ScriptDamage::set_damage)
            .register_get("damage", ScriptDamage::get_damage)
            .register_get("effective", ScriptDamage::effective)
            .register_type_with_name::<ScriptPokemon<ID>>("Pokemon")
            .register_fn("throw_move", ScriptPokemon::<ID>::throw_move::<R>)
            .register_fn("damage", ScriptPokemon::<ID>::get_damage::<R>)
            .register_get("hp", ScriptPokemon::<ID>::hp)
            .register_iterator::<Vec<ScriptPokemon<ID>>>()
            .register_type::<ScriptMove>()
            .register_get("category", ScriptMove::get_category)
            .register_get("type", ScriptMove::get_type)
            .register_get("crit_rate", ScriptMove::get_crit_rate)
            .register_type_with_name::<MoveCategory>("Category")
            .register_type_with_name::<PokemonType>("Type")
            .register_type::<MoveResult>()
            .register_type_with_name::<ScriptMoveResult<ID>>("Result")
            .register_fn("Miss", ScriptMoveResult::<ID>::miss)
            .register_fn("Damage", ScriptMoveResult::<ID>::damage)
            .register_fn("Drain", ScriptMoveResult::<ID>::heal);

        Self {
            scripts: Default::default(),
            engine,
        }
    }

    pub fn execute<
        'd,
        R: Rng + Clone + 'static,
        ID: Eq + Hash + Clone + 'static,
        P: Players<'d, ID, R>,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon<'d>>,
        targets: Vec<PokemonIdentifier<ID>>,
        players: &P,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, DefaultMoveError> {
        match self.scripts.get(&m.id) {
            Some(script) => {
                let mut scope = Scope::new();

                scope.push("random", ScriptRandom::new(random));
                scope.push("move", ScriptMove::new(m));
                scope.push("user", ScriptPokemon::<ID>::new(user));

                let targets = targets
                    .into_iter()
                    .flat_map(|id| (players.get(&id).map(|r| Indexed(id, r))))
                    .map(ScriptPokemon::new)
                    // .map(Dynamic::from)
                    .collect::<Vec<ScriptPokemon<ID>>>();

                scope.push("targets", targets);

                Ok(self
                    .engine
                    .eval_with_scope::<Array>(&mut scope, script)
                    .map_err(DefaultMoveError::Script)?
                    .into_iter()
                    .flat_map(Dynamic::try_cast::<ScriptMoveResult<ID>>)
                    .map(|r| r.0)
                    .collect::<Vec<Indexed<ID, MoveResult>>>())
            }
            None => return Err(DefaultMoveError::Missing),
        }
    }
}
