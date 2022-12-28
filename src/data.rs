use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattleData {
    /// constant
    pub versus: VersusType,
    /// constant
    pub active: usize,
    /// constant
    #[serde(default)]
    pub settings: BattleSettings,
    // add weather, etc
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VersusType {
    Wild,
    Trainer,
    GymLeader,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattleSettings {
    #[serde(default = "const_true")]
    pub allow_forfeit: bool,
}

impl VersusType {
    pub fn is_wild(&self) -> bool {
        matches!(self, Self::Wild)
    }

    pub fn is_trainer(&self) -> bool {
        !self.is_wild()
    }
}

impl Default for BattleData {
    fn default() -> Self {
        Self {
            versus: VersusType::Trainer,
            active: 1,
            settings: Default::default(),
        }
    }
}

impl Default for BattleSettings {
    fn default() -> Self {
        Self {
            allow_forfeit: true,
        }
    }
}

const fn const_true() -> bool {
    true
}
