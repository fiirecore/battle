use pokedex::{moves::Power, pokemon::Health, types::Effective};
use serde::{Deserialize, Serialize};

use super::Percent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DamageKind {
    Power(Power),
    PercentCurrent(Percent),
    PercentMax(Percent),
    Constant(Health),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ClientDamage<N> {
    Result(DamageResult<N>),
    Number(N),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageResult<N> {
    /// Inflicted damage
    pub damage: N,
    /// Whether the attack was effective
    pub effective: Effective,
    /// If the attack was a critical hit
    pub crit: bool,
}

impl<N> ClientDamage<N> {
    pub fn damage(self) -> N {
        match self {
            ClientDamage::Result(result) => result.damage,
            ClientDamage::Number(n) => n,
        }
    }

}

impl<N: Default> Default for DamageResult<N> {
    fn default() -> Self {
        Self {
            damage: Default::default(),
            effective: Effective::Ineffective,
            crit: false,
        }
    }
}

impl<N> From<N> for DamageResult<N> {
    fn from(damage: N) -> Self {
        Self {
            damage,
            effective: Effective::Effective,
            crit: false,
        }
    }
}