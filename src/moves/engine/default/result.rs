use rhai::INT;

use crate::moves::{target::TargetLocation, MoveResult};

use super::{damage::ScriptDamage, pokemon::ScriptPokemon};

#[derive(Clone, Copy)]
pub struct ScriptMoveResult(pub TargetLocation, pub MoveResult);

impl ScriptMoveResult {
    pub fn new(pokemon: ScriptPokemon, result: MoveResult) -> Self {
        Self(pokemon.into(), result)
    }

    pub fn miss() -> ScriptMoveResult {
        ScriptMoveResult(TargetLocation::User, MoveResult::Miss)
    }

    pub fn damage(damage: ScriptDamage, pokemon: ScriptPokemon) -> ScriptMoveResult {
        ScriptMoveResult::new(pokemon, MoveResult::Damage(damage.into()))
    }

    // pub const fn Status(effect: StatusEffect) -> MoveResult { MoveResult::Status(effect) }

    pub fn heal(heal: INT, pokemon: ScriptPokemon) -> ScriptMoveResult {
        ScriptMoveResult::new(pokemon, MoveResult::Heal(heal as _))
    }
}
