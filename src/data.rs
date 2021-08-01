use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattleData {
    pub type_: BattleType,
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
