use rhai::INT;

use crate::{moves::engine::MoveResult, pokemon::PokemonIndex};

use super::{damage::ScriptDamage, pokemon::ScriptPokemon};

#[derive(Clone, Copy)]
pub struct ScriptMoveResult<ID>(pub Option<PokemonIndex<ID>>, pub MoveResult);

impl<ID> ScriptMoveResult<ID> {
    pub fn new(pokemon: ScriptPokemon<ID>, result: MoveResult) -> Self {
        Self(Some(pokemon.into()), result)
    }

    pub fn miss() -> ScriptMoveResult<ID> {
        ScriptMoveResult(None, MoveResult::Miss)
    }

    pub fn damage(damage: ScriptDamage, pokemon: ScriptPokemon<ID>) -> ScriptMoveResult<ID> {
        ScriptMoveResult::new(pokemon, MoveResult::Damage(damage.into()))
    }

    // pub const fn Status(effect: StatusEffect) -> MoveResult { MoveResult::Status(effect) }

    pub fn heal(heal: INT, pokemon: ScriptPokemon<ID>) -> ScriptMoveResult<ID> {
        ScriptMoveResult::new(pokemon, MoveResult::Heal(heal as _))
    }
}
