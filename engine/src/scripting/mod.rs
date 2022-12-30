use core::{fmt::Debug, hash::Hash};
use std::error::Error;

// use std::error::Error;

use hashbrown::HashMap;
use rand::Rng;
use rhai::{
    packages::{BasicArrayPackage, Package},
    Engine, EvalAltResult, ParseError, Scope,
};

use battle::{
    data::BattleData,
    engine::{ActionResult, PlayerQuery},
    moves::{BattleMove, MoveCategory},
    pokedex::{item::ItemId, moves::MoveId},
    pokemon::{Indexed, TeamIndex},
    select::PublicAction,
};

type Scripts<ID> = HashMap<ID, String>;

pub type MoveScripts = Scripts<MoveId>;
pub type ItemScripts = Scripts<ItemId>;

mod moves;
pub use moves::*;

pub trait ScriptingEngine<ID, T> {
    /// Current battle data
    type Data: Default;

    type ExecutionError: Error;

    fn execute_move(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        m: &BattleMove,
        user: &TeamIndex<ID>,
        targets: Vec<TeamIndex<ID>>,
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError>;

    fn execute_item(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        item: &ItemId,
        user: &ID,
        target: TeamIndex<ID>,
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError>;
}

pub struct RhaiScriptingEngine {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub moves: MoveScripts,
    pub items: ItemScripts,
}

impl RhaiScriptingEngine {
    pub fn new<ID: Clone + Send + Sync + 'static, R: Rng + Clone + Send + Sync + 'static>() -> Self
    {
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
            .register_type::<ActionResult>()
            .register_type_with_name::<ScriptActionResult<ID>>("Result")
            .register_fn("Miss", ScriptActionResult::<ID>::miss)
            .register_fn("Damage", ScriptActionResult::<ID>::damage)
            .register_fn("Ailment", ScriptActionResult::<ID>::ailment)
            .register_fn("Drain", ScriptActionResult::<ID>::heal);

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

impl<ID: Eq + Hash + Clone + Send + Sync + 'static, T> ScriptingEngine<ID, T>
    for RhaiScriptingEngine
{
    type ExecutionError = RhaiScriptError;

    type Data = ();

    fn execute_move(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        m: &BattleMove,
        user: &TeamIndex<ID>,
        targets: Vec<TeamIndex<ID>>,
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError> {
        match self.moves.get(&m.id) {
            Some(script) => {
                use rhai::*;

                let mut scope = self.scope.clone();

                scope.push("random", ScriptRandom::new(random));

                let ast = self
                    .engine
                    .compile_with_scope(&mut scope, script)
                    .map_err(|err| RhaiScriptError::Parse(m.id, err))?;

                let mut iter = players.iter_mut();

                let targets = targets
                    .into_iter()
                    .flat_map(|id| {
                        iter.find(|p| p.id() == id.team())
                            .and_then(|p| p.party.active_mut(id.index()))
                            .map(|r| ScriptPokemon::new(Indexed(id, r)))
                    })
                    .collect::<Vec<ScriptPokemon<ID>>>();

                let p = players.get_mut(user.team()).unwrap().party.active_mut(user.index()).unwrap();

                let result: Array = self.engine.call_fn(
                    &mut scope,
                    &ast,
                    "use_move",
                    (ScriptMove::new(m), ScriptPokemon::<ID>::new(Indexed(user.clone(), p)), targets),
                )?;

                let result = result
                    .into_iter()
                    .flat_map(Dynamic::try_cast::<ScriptActionResult<ID>>)
                    .map(|r| r.0)
                    .collect::<Vec<Indexed<ID, ActionResult>>>();

                let mut actions = Vec::new();

                for action in result {
                    crate::run_action(action, battle, user, &mut actions, players);
                }

                Ok(actions)
            }
            None => Err(RhaiScriptError::Missing(m.id)),
        }
    }

    fn execute_item(
        &self,
        _data: &mut Self::Data,
        _random: &mut (impl Rng + Clone + Send + Sync + 'static),
        _battle: &mut BattleData,
        _item: &ItemId,
        _user: &ID,
        _target: TeamIndex<ID>,
        _players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError> {
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
            Self::Parse(id, err) => write!(
                f,
                "Cannot parse move script for {} with error {}",
                &id.0, err
            ),
            Self::Evaluate(err) => core::fmt::Display::fmt(err, f),
            Self::Missing(id) => write!(f, "Could not find move script with id {}", &id.0),
            Self::Unimplemented => write!(f, "Unimplemented feature!"),
        }
    }
}
