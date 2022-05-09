use core::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
};
use enum_map::Enum;
use serde::{Deserialize, Serialize};

use pokedex::pokemon::stat::{BaseStat, StatSet, StatType};

pub type Stage = i8;

#[derive(Debug, Clone, Copy, Enum, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleStatType {
    Basic(StatType),
    Accuracy,
    Evasion,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct StatStages(StatSet<BattleStatType, Stage>);

impl StatStages {
    pub fn can_change(&self, stat: BattleStatType, stage: Stage) -> bool {
        self[stat].abs() + stage < 6
    }

    pub fn change_stage(&mut self, stat: BattleStatType, stage: Stage) {
        self[stat] += stage;
    }

    pub fn mult(base: BaseStat, stage: Stage) -> BaseStat {
        base * (2.max(2 + stage) as BaseStat) / (2.max(2 - stage) as BaseStat)
    }
}

impl Deref for StatStages {
    type Target = StatSet<BattleStatType, Stage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StatStages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for BattleStatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BattleStatType::Basic(stat) => Debug::fmt(stat, f),
            stat => Debug::fmt(stat, f),
        }
    }
}
