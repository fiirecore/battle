use core::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use pokedex::pokemon::stat::{Stage, StatSet, BaseStat, StatStage, StatType};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatStages {
    pub basic: StatSet<Stage>,
    pub accuracy: Stage,
    pub evasion: Stage,
}

impl StatStages {

    pub fn get(&self, stat: StatType) -> Stage {
        match stat {
            StatType::Accuracy => self.accuracy,
            StatType::Evasion => self.evasion,
            StatType::Health => self.hp,
            StatType::Attack => self.atk,
            StatType::Defense => self.def,
            StatType::SpAttack => self.sp_atk,
            StatType::SpDefense => self.sp_def,
            StatType::Speed => self.speed,
        }
    }

    pub fn get_mut(&mut self, stat: StatType) -> &mut Stage {
        match stat {
            StatType::Accuracy => &mut self.accuracy,
            StatType::Evasion => &mut self.evasion,
            StatType::Health => &mut self.hp,
            StatType::Attack => &mut self.atk,
            StatType::Defense => &mut self.def,
            StatType::SpAttack => &mut self.sp_atk,
            StatType::SpDefense => &mut self.sp_def,
            StatType::Speed => &mut self.speed,
        }
    }

    pub fn can_change(&self, stat: &StatStage) -> bool {
        self.get(stat.stat).abs() + stat.stage < 6
    }

    pub fn change_stage(&mut self, stat: StatStage) {
        *self.get_mut(stat.stat) += stat.stage;
    }

    pub fn mult(base: BaseStat, stage: Stage) -> BaseStat {
        base * (2.max(2 + stage) as BaseStat) / (2.max(2 - stage) as BaseStat)
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