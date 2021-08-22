use rhai::INT;

use pokedex::{pokemon::Health, types::Effective};

use crate::moves::usage::DamageResult;

#[derive(Clone, Copy)]
pub struct ScriptDamage(DamageResult<INT>);

impl ScriptDamage {
    pub fn set_damage(&mut self, damage: INT) {
        self.0.damage = damage;
    }
    pub fn get_damage(&mut self) -> INT {
        self.0.damage
    }
    pub fn effective(&mut self) -> Effective {
        self.0.effective
    }
}

impl From<DamageResult<Health>> for ScriptDamage {
    fn from(result: DamageResult<Health>) -> Self {
        Self(DamageResult {
            damage: result.damage as _,
            effective: result.effective,
            crit: result.crit,
        })
    }
}

impl Into<DamageResult<Health>> for ScriptDamage {
    fn into(self) -> DamageResult<Health> {
        DamageResult {
            damage: self.0.damage as _,
            effective: self.0.effective,
            crit: self.0.crit,
        }
    }
}