use pokedex::{moves::Power, pokemon::Health, types::Effective};
use serde::{Deserialize, Serialize};

use super::Percent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum DamageKind {
    Power(Power),
    PercentCurrent(Percent),
    PercentMax(Percent),
    Constant(Health),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageResult<INT> {
    /// Inflicted damage
    pub damage: INT,
    /// Whether the attack was effective
    pub effective: Effective,
    /// If the attack was a critical hit
    pub crit: bool,
}

impl<INT: Default> Default for DamageResult<INT> {
    fn default() -> Self {
        Self {
            damage: Default::default(),
            effective: Effective::Ineffective,
            crit: false,
        }
    }
}
