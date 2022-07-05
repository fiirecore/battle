use hashbrown::HashMap;

use rand::Rng;

use pokedex::{item::ItemId, moves::MoveId};

pub mod item;
use self::item::BattleItemExecution;

pub mod moves;
use self::moves::MoveExecution;

pub mod scripting;
use self::scripting::ScriptingEngine;

#[cfg(feature = "default_engine_scripting")]
pub mod default_scripting;

pub(crate) mod prelude {

    pub use super::{DefaultEngine, EngineItems, EngineMoves};

    #[cfg(feature = "default_engine_scripting")]
    pub use super::scripting::*;
}

pub type EngineItems = HashMap<ItemId, BattleItemExecution>;
pub type EngineMoves = HashMap<MoveId, MoveExecution>;

#[cfg(feature = "default_engine_scripting")]
pub type DefaultEngine = DefaultBattleEngine<default_scripting::RhaiScriptingEngine>;

#[cfg(not(feature = "default_engine_scripting"))]
pub type DefaultEngine = DefaultBattleEngine<scripting::DefaultScriptEngine>;

pub struct DefaultBattleEngine<S: ScriptingEngine> {
    pub items: EngineItems,
    pub moves: EngineMoves,
    pub scripting: S,
}

#[cfg(feature = "default_engine_scripting")]
impl DefaultBattleEngine<default_scripting::RhaiScriptingEngine> {
    pub fn new<ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        Self {
            items: Default::default(),
            moves: Default::default(),
            scripting: default_scripting::RhaiScriptingEngine::new::<ID, R>(),
        }
    }
}