use core::{fmt::Debug, hash::Hash, ops::Deref};

use std::error::Error;

use rand::Rng;

use pokedex::{
    item::Item,
    moves::{Move, MoveId},
    pokemon::Pokemon,
};

use crate::{
    engine::{pokemon::throw_move, BattlePokemon, Players},
    moves::engine::{MoveEngine, MoveResult},
    pokemon::{Indexed, PokemonIdentifier},
};

use super::{DefaultEngine, ScriptError};

#[cfg(feature = "default_engine_scripting")]
pub mod scripting;

mod execution;
pub use execution::*;

impl MoveEngine for DefaultEngine {
    type Error = MoveError;

    fn execute<
        ID: Clone + Hash + Eq + 'static + Debug,
        R: Rng + Clone + 'static,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
        PLR: Players<ID, P, M, I>,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon<P, M, I>>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error> {
        match self.moves.get(&m.id) {
            Some(usage) => {
                let targets = players.create_targets(&user.0, m, targeting, random);

                match &usage {
                    MoveExecution::Actions(actions) => {
                        let mut results = Vec::new();
                        for target_id in targets {
                            match players.get(&target_id) {
                                Some(target) => match throw_move(random, m.accuracy) {
                                    true => {
                                        results.reserve(usage.size());
                                        move_usage(
                                            &user,
                                            random,
                                            &mut results,
                                            actions,
                                            m,
                                            Indexed(target_id, target),
                                        );
                                    }
                                    false => {
                                        results.push(Indexed(user.0.clone(), MoveResult::Miss))
                                    }
                                },
                                None => (),
                            }
                        }
                        Ok(results)
                    }
                    MoveExecution::Script => {
                        #[cfg(feature = "default_engine_scripting")]
                        return self
                            .scripting
                            .execute_move(random, m, user, targets, players);
                        #[cfg(not(feature = "default_engine_scripting"))]
                        return Err(MoveError::Script(ScriptError::default()));
                    }
                    MoveExecution::None => Err(MoveError::Missing(m.id)),
                }
            }
            None => Err(MoveError::Missing(m.id)),
        }
    }
}

#[derive(Debug)]
pub enum MoveError {
    Script(ScriptError),
    Missing(MoveId),
    NoTarget,
}

impl MoveError {
    #[cfg(feature = "default_engine_scripting")]
    fn script(error: Box<quad_compat_rhai::EvalAltResult>) -> Self {
        Self::Script(ScriptError::from(error))
    }
}

impl Error for MoveError {}

impl core::fmt::Display for MoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "default_engine_scripting")]
            Self::Script(err) => core::fmt::Display::fmt(err, f),
            other => core::fmt::Debug::fmt(other, f),
        }
    }
}
