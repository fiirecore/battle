use hashbrown::HashMap;

use rand::Rng;

use pokedex::{item::ItemId, moves::MoveId};

pub mod item;
use self::item::BattleItemExecution;

pub mod moves;
use self::moves::MoveExecution;

#[cfg(feature = "default_engine_scripting")]
pub mod scripting;

pub(crate) mod prelude {

    pub use super::{DefaultEngine, EngineItems, EngineMoves};

    #[cfg(feature = "default_engine_scripting")]
    pub use super::scripting::*;
}

pub type EngineItems = HashMap<ItemId, BattleItemExecution>;
pub type EngineMoves = HashMap<MoveId, MoveExecution>;

pub struct DefaultEngine {
    pub items: EngineItems,
    pub moves: EngineMoves,
    #[cfg(feature = "default_engine_scripting")]
    pub scripting: scripting::ScriptingEngine,
}

impl DefaultEngine {
    pub fn new<ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        Self {
            items: Default::default(),
            moves: Default::default(),
            #[cfg(feature = "default_engine_scripting")]
            scripting: scripting::ScriptingEngine::new::<ID, R>(),
        }
    }
}

#[derive(Debug)]
pub enum ScriptError {
    #[cfg(feature = "default_engine_scripting")]
    Error(Box<rhai::EvalAltResult>),
    #[cfg(not(feature = "default_engine_scripting"))]
    NoScriptEngine,
}

#[cfg(not(feature = "default_engine_scripting"))]
impl Default for ScriptError {
    fn default() -> Self {
        Self::NoScriptEngine
    }
}

#[cfg(feature = "default_engine_scripting")]
impl From<Box<rhai::EvalAltResult>> for ScriptError {
    fn from(r: Box<rhai::EvalAltResult>) -> Self {
        Self::Error(r)
    }
}

impl core::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "default_engine_scripting")]
            Self::Error(err) => core::fmt::Display::fmt(err, f),
            #[cfg(not(feature = "default_engine_scripting"))]
            Self::NoScriptEngine => write!(
                f,
                "No scripting engine has been provided to the default move/item engine!"
            ),
        }
    }
}
