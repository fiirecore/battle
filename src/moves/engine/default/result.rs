use crate::moves::{target::TargetLocation, MoveResult};

use super::pokemon::ScriptPokemon;

#[derive(Clone, Copy)]
pub struct ScriptMoveResult(pub TargetLocation, pub MoveResult);

impl ScriptMoveResult {
    pub fn new(pokemon: ScriptPokemon, result: MoveResult) -> Self {
        Self(pokemon.into(), result)
    }
}
