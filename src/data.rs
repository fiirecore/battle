use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattleData {
    pub type_: BattleType,
    #[serde(default)]
    pub settings: BattleSettings,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BattleType {
    Wild,
    Trainer,
    GymLeader,
}

impl Default for BattleType {
    fn default() -> Self {
        Self::Wild
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattleSettings {
    #[serde(default = "const_true")]
    pub allow_forfeit: bool,
}

impl Default for BattleSettings {
    fn default() -> Self {
        Self { allow_forfeit: true }
    }
}

const fn const_true() -> bool {
    true
}