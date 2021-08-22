use serde::{Deserialize, Serialize};

use pokedex::ailment::{Ailment, AilmentLength};

use crate::pokemon::battle::stat::{BattleStatType, Stage};

mod damage;
pub use damage::*;

mod result;
pub use result::*;

pub mod target;

pub mod engine;

pub type CriticalRate = u8;
pub type Critical = bool;
/// 0 through 100
pub type Percent = u8;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoveUsage {
    /// What the move does in battle
    pub execute: MoveExecution,

    /// If the move makes contact with the target.
    #[serde(default)]
    pub contact: bool,

    #[serde(default)]
    pub crit_rate: CriticalRate,
}

// #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
// pub struct Targets<T> {
//     pub user: T,
//     pub target: T,
// }

pub type MoveScriptId = tinystr::TinyStr16;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MoveExecution {
    /// Load a vector of actions
    Actions(Vec<MoveAction>),
    /// Use a script defined in the instance of the object that uses this
    Script(MoveScriptId),
    /// Placeholder to show that object does not have a defined use yet.
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum MoveAction {
    Damage(DamageKind),
    Ailment(Ailment, AilmentLength, Percent),
    Drain(DamageKind, i8),
    Stat(BattleStatType, Stage),
    Flinch,
    Chance(Vec<Self>, Percent),
}

impl MoveExecution {
    pub fn len(&self) -> usize {
        match self {
            Self::Actions(actions) => actions.iter().map(MoveAction::len).sum(),
            Self::Script(..) | Self::None => 1,
        }
    }
}

impl MoveAction {
    pub fn len(&self) -> usize {
        match self {
            Self::Chance(uses, ..) => uses.iter().map(Self::len).sum(),
            _ => 1,
        }
    }
}
