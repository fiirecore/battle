use serde::{Deserialize, Serialize};

use pokedex::ailment::{Ailment, AilmentLength};

use crate::pokemon::battle::stat::{BattleStatType, Stage};

mod damage;
pub use damage::*;

mod result;
pub use result::*;

mod target;
pub use target::*;

pub mod script;

pub type CriticalRate = u8;
pub type Critical = bool;
pub type Percent = u8; // 0 to 100

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoveUsage {
    pub pokemon: MoveUsageKind,

    #[serde(default)]
    pub target: MoveTarget,

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MoveUsageKind {
    /// Load a vector of actions
    Actions(Vec<MoveAction>),
    /// Use a script defined in the instance of the object that uses this
    Script,
    /// Placeholder to show that object does not have a defined use yet.
    Todo,
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

impl MoveUsageKind {
    pub fn len(&self) -> usize {
        match self {
            Self::Actions(actions) => actions.iter().map(MoveAction::len).sum(),
            Self::Script => 0,
            Self::Todo => 1,
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
