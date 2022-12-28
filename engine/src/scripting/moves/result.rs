use rhai::INT;

use battle::{engine::ActionResult, pokemon::Indexed, select::ClientMoveAction};

use super::{damage::ScriptDamage, pokemon::ScriptPokemon, LiveScriptAilment};

#[derive(Clone, Copy)]
pub struct ScriptActionResult<ID: Clone + Send + Sync + 'static>(pub Indexed<ID, ClientMoveAction>);

impl<ID: Clone + Send + Sync + 'static> ScriptActionResult<ID> {
    pub fn new(pokemon: ScriptPokemon<ID>, result: ActionResult) -> Self {
        let action = ClientMoveAction::new(&*pokemon, result);
        Self(Indexed(pokemon.position().clone(), action))
    }

    pub fn damage(pokemon: ScriptPokemon<ID>, damage: ScriptDamage) -> Self {
        Self::new(pokemon, ActionResult::Damage(damage.into()))
    }

    pub fn heal(pokemon: ScriptPokemon<ID>, heal: INT) -> Self {
        Self::new(pokemon, ActionResult::Heal(heal as _))
    }

    pub fn ailment(pokemon: ScriptPokemon<ID>, ailment: LiveScriptAilment) -> Self {
        Self::new(pokemon, ActionResult::Ailment(ailment.0))
    }

    // pub const fn Status(effect: StatusEffect) -> MoveResult { MoveResult::Status(effect) }

    pub fn miss(pokemon: ScriptPokemon<ID>) -> Self {
        Self::new(pokemon, ActionResult::Miss)
    }
}
