use rhai::INT;

use battle::{
    moves::DamageResult,
    pokedex::{pokemon::Health, types::Effective},
};

#[derive(Debug, Clone, Copy)]
pub struct ScriptDamage(DamageResult<INT>);

impl ScriptDamage {
    pub fn with_damage(damage: INT) -> Self {
        Self(DamageResult::from(damage))
    }

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

impl From<DamageResult<INT>> for ScriptDamage {
    fn from(result: DamageResult<INT>) -> Self {
        Self(result)
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

impl From<ScriptDamage> for DamageResult<Health> {
    fn from(s: ScriptDamage) -> Self {
        Self {
            damage: s.0.damage as _,
            effective: s.0.effective,
            crit: s.0.crit,
        }
    }
}
