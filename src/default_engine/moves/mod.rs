use core::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::Hash,
    ops::Deref,
};

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

use super::{DefaultBattleEngine, ScriptingEngine};

mod execution;
pub use execution::*;

impl<S: ScriptingEngine> MoveEngine for DefaultBattleEngine<S> {
    type Error = MoveError<S::Error>;

    fn execute<
        ID: Clone + Hash + Eq + 'static + Debug,
        R: Rng + Clone + 'static,
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
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
                            .execute_move(random, m, user, targets, players).map_err(MoveError::Script);
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
pub enum MoveError<S: Error = NoScriptError> {
    Script(S),
    Missing(MoveId),
    NoTarget,
}

impl<S: Error> Error for MoveError<S> {}

impl<S: Error> Display for MoveError<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Script(err) => Display::fmt(err, f),
            other => Debug::fmt(other, f),
        }
    }
}

#[derive(Debug)]
pub enum NoScriptError {
    NoScriptEngine,
}

impl Error for NoScriptError {}

impl Display for NoScriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "No scripting engine!")
    }
}
