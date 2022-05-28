use core::{hash::Hash, ops::Deref};

use rhai::{Array, Dynamic, Scope};
use rand::Rng;

use pokedex::{item::Item, moves::Move, pokemon::Pokemon};

pub use pokedex::{moves::MoveCategory, types::PokemonType};

mod damage;
mod moves;
mod pokemon;
mod random;
mod result;

pub use damage::*;
pub use moves::*;
pub use pokemon::*;
pub use random::*;
pub use result::*;

use crate::{
    engine::{BattlePokemon, Players},
    moves::engine::MoveResult,
    pokemon::{Indexed, PokemonIdentifier},
};

use super::{super::scripting::ScriptingEngine, MoveError};

impl ScriptingEngine {
    pub fn execute_move<
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
        R: Rng + Clone + 'static,
        ID: Eq + Hash + Clone + 'static + core::fmt::Debug,
        PLR: Players<ID, P, M, I>,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon<P, M, I>>,
        targets: Vec<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, MoveError> {
        match self.moves.get(&m.id) {
            Some(script) => {
                let mut scope = Scope::new();

                scope.push("random", ScriptRandom::new(random));
                scope.push("move", ScriptMove::new(m));
                scope.push("user", ScriptPokemon::<ID>::new(user));

                let targets = targets
                    .into_iter()
                    .flat_map(|id| (players.get(&id).map(|r| Indexed(id, r))))
                    .map(ScriptPokemon::new)
                    .collect::<Vec<ScriptPokemon<ID>>>();

                scope.push("targets", targets);

                Ok(self
                    .engine
                    .eval_with_scope::<Array>(&mut scope, script)
                    .map_err(MoveError::script)?
                    .into_iter()
                    .flat_map(Dynamic::try_cast::<ScriptMoveResult<ID>>)
                    .map(|r| r.0)
                    .collect::<Vec<Indexed<ID, MoveResult>>>())
            }
            None => Err(MoveError::Missing(m.id)),
        }
    }
}
