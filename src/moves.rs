use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::{Ailment, AilmentLength, LiveAilment},
    pokemon::{Health, Experience, Level},
    moves::MoveId,
    item::ItemId,
};

use crate::pokemon::battle::stat::{BattleStatType, Stage};

pub mod damage;
pub mod engine;
pub mod target;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BattleMove {
    Move(usize, target::MoveTargetInstance),
    UseItem(ItemId, usize),
    Switch(usize),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMove {
    Move(MoveId, Vec<(target::TargetLocation, ClientMoveAction)>),
    Switch(usize),
    UseItem(ItemId, usize),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClientMoveAction {
    /// This contains the percent HP the pokemon was left at, how effective the attack was, and if it was a critical hit.
    /// A Pokemon faints when it's hp is set to 0.0
    SetDamage(damage::DamageResult<f32>),
    /// A Pokemon faints when it's hp is set to 0.0
    SetHP(f32),
    AddStat(BattleStatType, Stage),
    Ailment(LiveAilment),
    Miss,

    SetExp(Experience, Level),

    Error,
}

pub type Critical = bool;
/// 0 through 100
pub type Percent = u8;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MoveExecution {
    /// Load a vector of actions
    Actions(Vec<MoveUse>),
    /// Use a script defined in the instance of the object that uses this
    Script,
    /// Placeholder to show that object does not have a defined use yet.
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum MoveUse {
    Damage(damage::DamageKind),
    Ailment(Ailment, AilmentLength, Percent),
    Drain(damage::DamageKind, i8),
    Stat(BattleStatType, Stage),
    Flinch,
    Chance(Vec<Self>, Percent),
}

impl MoveExecution {
    pub fn len(&self) -> usize {
        match self {
            Self::Actions(actions) => actions.iter().map(MoveUse::len).sum(),
            Self::Script | Self::None => 1,
        }
    }
}

impl MoveUse {
    pub fn len(&self) -> usize {
        match self {
            Self::Chance(uses, ..) => uses.iter().map(Self::len).sum(),
            Self::Drain(..) => 2,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Damage(damage::DamageResult<Health>),
    Heal(i16),
    Ailment(LiveAilment),
    Stat(BattleStatType, Stage),
    Flinch,
    Miss,
    Error,
}