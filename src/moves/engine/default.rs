use hashbrown::HashMap;
use rand::Rng;
use std::error::Error;

use pokedex::{
    moves::{Move, MoveCategory, MoveId},
    types::PokemonType,
};

use rhai::{plugin::*, Array, Dynamic, Engine, Scope, AST, INT};

use crate::{
    moves::{
        damage::DamageResult, engine::MoveEngine, target::TargetLocation, MoveExecution, MoveResult,
    },
    pokemon::battle::BattlePokemon,
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
pub use result::*;

pub type Moves = HashMap<MoveId, MoveExecution>;
pub type Scripts = HashMap<MoveId, AST>;

pub struct DefaultMoveEngine {
    pub moves: Moves,
    pub scripts: Scripts,
    pub engine: Engine,
}

impl DefaultMoveEngine {
    pub fn new<R: Rng + Clone + 'static>() -> Self {
        let mut engine = Engine::new_raw();

        engine
            .register_type_with_name::<ScriptRandom<R>>("Random")
            .register_type_with_name::<DamageResult<INT>>("Damage")
            .register_set("damage", ScriptDamage::set_damage)
            .register_get("damage", ScriptDamage::get_damage)
            .register_get("effective", ScriptDamage::effective)
            .register_type_with_name::<ScriptPokemon>("Pokemon")
            .register_fn("damage", ScriptPokemon::get_damage::<R>)
            .register_get("hp", ScriptPokemon::hp)
            .register_type::<ScriptMove>()
            .register_fn("try_hit", ScriptMove::try_hit::<R>)
            .register_get("category", ScriptMove::get_category)
            .register_get("type", ScriptMove::get_type)
            .register_get("crit_rate", ScriptMove::get_crit_rate)
            .register_type_with_name::<MoveCategory>("Category")
            .register_type_with_name::<PokemonType>("Type")
            .register_type::<MoveResult>()
            .register_type_with_name::<ScriptMoveResult>("Result")
            .register_fn("miss", ScriptMoveResult::miss)
            .register_fn("damage", ScriptMoveResult::damage)
            .register_fn("drain", ScriptMoveResult::heal);

        Self {
            moves: Default::default(),
            scripts: Default::default(),
            engine,
        }
    }
}

impl MoveEngine for DefaultMoveEngine {
    type Error = DefaultMoveError;

    fn execute<'d, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        m: &Move,
        user: &BattlePokemon<'d>,
        targets: Vec<(TargetLocation, &BattlePokemon<'d>)>,
    ) -> Result<Vec<(TargetLocation, MoveResult)>, Self::Error> {
        match self.moves.get(&m.id) {
            Some(usage) => {
                let mut results = Vec::new();

                match &usage {
                    MoveExecution::Actions(actions) => {
                        for target in targets {
                            match m.try_hit(random) {
                                true => {
                                    results.reserve(usage.len());
                                    user.move_usage(random, &mut results, actions, m, target);
                                }
                                false => results.push((TargetLocation::User, MoveResult::Miss)),
                            }
                        }
                    }
                    MoveExecution::Script => match self.scripts.get(&m.id) {
                        Some(script) => {
                            let mut scope = Scope::new();

                            scope.push("random", ScriptRandom::new(random));
                            scope.push("move", ScriptMove::new(m));
                            scope.push("user", ScriptPokemon::new((TargetLocation::User, user)));

                            let targets = targets
                                .into_iter()
                                .map(ScriptPokemon::new)
                                .map(Dynamic::from)
                                .collect::<Array>();

                            scope.push("targets", targets);

                            results.extend(
                                self.engine
                                    .eval_ast_with_scope::<Array>(&mut scope, script)
                                    .map_err(DefaultMoveError::Script)?
                                    .into_iter()
                                    .flat_map(Dynamic::try_cast::<ScriptMoveResult>)
                                    .map(|r| (r.0, r.1))
                                    .collect::<Vec<(TargetLocation, MoveResult)>>(),
                            );
                        }
                        None => return Err(DefaultMoveError::Missing),
                    },
                    MoveExecution::None => return Err(DefaultMoveError::Missing),
                }

                Ok(results)
            }
            None => Err(DefaultMoveError::Missing),
        }
    }
}

#[derive(Debug)]
pub enum DefaultMoveError {
    Script(Box<EvalAltResult>),
    Missing,
}

impl Error for DefaultMoveError {}

impl core::fmt::Display for DefaultMoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Script(err) => core::fmt::Display::fmt(err, f),
            other => core::fmt::Debug::fmt(other, f),
        }
    }
}
