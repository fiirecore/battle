use rhai::INT;

use crate::{
    host::engine::MoveResult,
    pokemon::{Indexed, PokemonIdentifier},
};

use super::{damage::ScriptDamage, pokemon::ScriptPokemon};

#[derive(Clone, Copy)]
pub struct ScriptMoveResult<ID>(pub Indexed<ID, MoveResult>);

impl<ID> ScriptMoveResult<ID> {
    pub fn new(pokemon: ScriptPokemon<ID>, result: MoveResult) -> Self {
        Self(Indexed(pokemon.into(), result))
    }

    pub fn miss(user: PokemonIdentifier<ID>) -> ScriptMoveResult<ID> {
        ScriptMoveResult(Indexed(user, MoveResult::Miss))
    }

    pub fn damage(damage: ScriptDamage, pokemon: ScriptPokemon<ID>) -> ScriptMoveResult<ID> {
        ScriptMoveResult::new(pokemon, MoveResult::Damage(damage.into()))
    }

    // pub const fn Status(effect: StatusEffect) -> MoveResult { MoveResult::Status(effect) }

    pub fn heal(heal: INT, pokemon: ScriptPokemon<ID>) -> ScriptMoveResult<ID> {
        ScriptMoveResult::new(pokemon, MoveResult::Heal(heal as _))
    }
}
