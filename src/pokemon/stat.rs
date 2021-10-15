use core::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
};
use serde::{Deserialize, Serialize};

use pokedex::pokemon::stat::{BaseStat, StatSet, StatType};

pub type Stage = i8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleStatType {
    Basic(StatType),
    Accuracy,
    Evasion,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatStages {
    pub basic: StatSet<Stage>,
    pub accuracy: Stage,
    pub evasion: Stage,
}

impl StatStages {
    pub fn get(&self, stat: BattleStatType) -> Stage {
        match stat {
            BattleStatType::Basic(stat) => match stat {
                StatType::Health => self.hp,
                StatType::Attack => self.atk,
                StatType::Defense => self.def,
                StatType::SpAttack => self.sp_atk,
                StatType::SpDefense => self.sp_def,
                StatType::Speed => self.speed,
            },
            BattleStatType::Accuracy => self.accuracy,
            BattleStatType::Evasion => self.evasion,
        }
    }

    pub fn get_mut(&mut self, stat: BattleStatType) -> &mut Stage {
        match stat {
            BattleStatType::Basic(stat) => match stat {
                StatType::Health => &mut self.hp,
                StatType::Attack => &mut self.atk,
                StatType::Defense => &mut self.def,
                StatType::SpAttack => &mut self.sp_atk,
                StatType::SpDefense => &mut self.sp_def,
                StatType::Speed => &mut self.speed,
            },
            BattleStatType::Accuracy => &mut self.accuracy,
            BattleStatType::Evasion => &mut self.evasion,
        }
    }

    pub fn can_change(&self, stat: BattleStatType, stage: Stage) -> bool {
        self.get(stat).abs() + stage < 6
    }

    pub fn change_stage(&mut self, stat: BattleStatType, stage: Stage) {
        *self.get_mut(stat) += stage;
    }

    pub fn mult(base: BaseStat, stage: Stage) -> BaseStat {
        base * (2.max(2 + stage) as BaseStat) / (2.max(2 - stage) as BaseStat)
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

impl Deref for StatStages {
    type Target = StatSet<Stage>;

    fn deref(&self) -> &Self::Target {
        &self.basic
    }
}

impl DerefMut for StatStages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.basic
    }
}
