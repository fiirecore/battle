use alloc::{boxed::Box, string::String, vec::Vec};
use core::{fmt::Debug, hash::Hash};

// use std::error::Error;

use hashbrown::HashMap;
use rand::Rng;
use rhai::{
    packages::{BasicArrayPackage, Package},
    Engine, EvalAltResult, ParseError, Scope,
};

use pokedex::{
    item::{Item, ItemId},
    moves::{Move, MoveId},
};

use crate::{
    engine::{BattlePokemon, ItemResult, MoveResult, Players},
    pokemon::{Indexed, PokemonIdentifier},
    prelude::BattleData,
};

use super::scripting::ScriptingEngine;

type Scripts<ID> = HashMap<ID, String>;

pub type MoveScripts = Scripts<MoveId>;
pub type ItemScripts = Scripts<ItemId>;

mod moves;
pub use moves::*;

pub struct RhaiScriptingEngine {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub moves: MoveScripts,
    pub items: ItemScripts,
}

impl RhaiScriptingEngine {
    pub fn new<ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        let mut engine = Engine::new();

        engine
            .register_global_module(BasicArrayPackage::new().as_shared_module())
            .register_type_with_name::<ScriptRandom<R>>("Random")
            .register_type_with_name::<ScriptDamage>("Damage")
            .register_fn("damage", ScriptDamage::with_damage)
            .register_set("damage", ScriptDamage::set_damage)
            .register_get("damage", ScriptDamage::get_damage)
            .register_get("effective", ScriptDamage::effective)
            .register_type_with_name::<ScriptAilmentEffect>("AilmentEffect")
            .register_type_with_name::<LiveScriptAilment>("LiveAilment")
            .register_fn("ailment", ScriptAilmentEffect::ailment)
            .register_fn("init", ScriptAilmentEffect::init::<R>)
            .register_fn("clear_ailment", LiveScriptAilment::clear_ailment)
            .register_type_with_name::<ScriptPokemon<ID>>("Pokemon")
            .register_iterator::<Vec<ScriptPokemon<ID>>>()
            .register_fn("throw_move", ScriptPokemon::<ID>::throw_move::<R>)
            // .register_fn("ailment_affects", ScriptPokemon::<ID>::ailment_affects)
            .register_fn("damage", ScriptPokemon::<ID>::get_damage::<R>)
            .register_get("hp", ScriptPokemon::<ID>::hp)
            .register_iterator::<Vec<ScriptPokemon<ID>>>()
            .register_type_with_name::<ScriptMove>("Move")
            .register_get("category", ScriptMove::get_category)
            .register_get("type", ScriptMove::get_type)
            .register_get("crit_rate", ScriptMove::get_crit_rate)
            .register_type_with_name::<MoveCategory>("Category")
            .register_type_with_name::<PokemonType>("Type")
            .register_type::<MoveResult>()
            .register_type_with_name::<ScriptMoveResult<ID>>("Result")
            .register_fn("Miss", ScriptMoveResult::<ID>::miss)
            .register_fn("Damage", ScriptMoveResult::<ID>::damage)
            .register_fn("Ailment", ScriptMoveResult::<ID>::ailment)
            .register_fn("Drain", ScriptMoveResult::<ID>::heal);

        engine.set_max_expr_depths(0, 0);
        // engine.set_optimization_level(rhai::OptimizationLevel::Full);

        let mut scope = Scope::new();

        scope.push_constant("CLEAR", LiveScriptAilment::clear_ailment());

        Self {
            items: Default::default(),
            moves: Default::default(),
            engine,
            scope,
        }
    }
}

impl ScriptingEngine for RhaiScriptingEngine {
    type Error = RhaiScriptError;

    fn execute_move<
        ID: Eq + Hash + Clone + 'static + Debug,
        R: Rng + Clone + 'static,
        PLR: Players<ID>,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon>,
        targets: Vec<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error> {
        match self.moves.get(&m.id) {
            Some(script) => {
                use rhai::*;

                let mut scope = self.scope.clone();

                scope.push("random", ScriptRandom::new(random));

                let ast = self
                    .engine
                    .compile_with_scope(&mut scope, script)
                    .map_err(|err| RhaiScriptError::Parse(m.id, err))?;

                let targets = targets
                    .into_iter()
                    .flat_map(|id| (players.get(&id).map(|r| Indexed(id, r))))
                    .map(ScriptPokemon::new)
                    .collect::<Vec<ScriptPokemon<ID>>>();

                let result: Array = self.engine.call_fn(
                    &mut scope,
                    &ast,
                    "use_move",
                    (ScriptMove::new(m), ScriptPokemon::<ID>::new(user), targets),
                )?;

                let result = result
                    .into_iter()
                    .flat_map(Dynamic::try_cast::<ScriptMoveResult<ID>>)
                    .map(|r| r.0)
                    .collect::<Vec<Indexed<ID, MoveResult>>>();

                Ok(result)
            }
            None => Err(RhaiScriptError::Missing(m.id)),
        }
    }

    fn execute_item<ID: PartialEq, R: Rng, PLR: Players<ID>>(
        &self,
        _battle: &BattleData,
        _random: &mut R,
        _item: &Item,
        _user: &ID,
        _target: PokemonIdentifier<ID>,
        _players: &mut PLR,
    ) -> Result<Vec<ItemResult>, Self::Error> {
        log::debug!("to - do: item scripting");
        Err(RhaiScriptError::Unimplemented)
    }
}

#[derive(Debug)]
pub enum RhaiScriptError {
    Parse(MoveId, ParseError),
    Evaluate(Box<EvalAltResult>),
    Missing(MoveId),
    Unimplemented,
}

impl std::error::Error for RhaiScriptError {}

impl From<Box<EvalAltResult>> for RhaiScriptError {
    fn from(r: Box<EvalAltResult>) -> Self {
        Self::Evaluate(r)
    }
}

impl core::fmt::Display for RhaiScriptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "default_engine_scripting")]
            Self::Parse(id, err) => write!(
                f,
                "Cannot parse move script for {} with error {}",
                &id.0, err
            ),
            #[cfg(feature = "default_engine_scripting")]
            Self::Evaluate(err) => core::fmt::Display::fmt(err, f),
            Self::Missing(id) => write!(f, "Could not find move script with id {}", &id.0),
            Self::Unimplemented => write!(f, "Unimplemented feature!"),
        }
    }
}
