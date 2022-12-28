use core::ops::DerefMut;

use battle::pokedex::ailment::*;
use rand::Rng;

use super::ScriptRandom;

#[derive(Debug, Clone, Copy)]
pub struct ScriptAilment<A>(pub Option<A>);

pub type ScriptAilmentEffect = ScriptAilment<AilmentEffect>;
pub type LiveScriptAilment = ScriptAilment<LiveAilment>;

impl ScriptAilmentEffect {
    pub fn ailment(ailment: Ailment, turns: AilmentLength) -> Self {
        Self(Some(AilmentEffect { ailment, turns }))
    }

    pub fn init<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        mut random: ScriptRandom<R>,
    ) -> LiveScriptAilment {
        let random = random.deref_mut();
        ScriptAilment(
            self.0
                .map(|ailment| ailment.turns.init(ailment.ailment, random)),
        )
    }
}

impl LiveScriptAilment {
    pub fn clear_ailment() -> LiveScriptAilment {
        Self(None)
    }
}
