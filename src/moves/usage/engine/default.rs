use hashbrown::HashMap;
use rand::Rng;
use std::error::Error;

use pokedex::moves::{Move, MoveId};

use crate::{
    moves::usage::{engine::MoveEngine, MoveExecution, MoveResult, MoveUsage, NoHitResult, MoveScriptId},
    pokemon::battle::BattlePokemon,
};

use self::script::RhaiMoveScriptEngine;

pub mod script;

pub type Moves = HashMap<MoveId, MoveUsage>;

pub type RhaiMoveEngine<R> = DefaultMoveEngine<RhaiMoveScriptEngine<R>>;

pub struct DefaultMoveEngine<S: MoveScriptEngine + core::fmt::Debug> {
    pub moves: HashMap<MoveId, MoveUsage>,
    pub scripts: S,
}

pub trait MoveScriptEngine {
    type Error: Error;

    fn execute<'a, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        used_move: &Move,
        usage: &MoveUsage,
        id: &MoveScriptId,
        user: &BattlePokemon<'a>,
        target: &BattlePokemon<'a>,
    ) -> Result<Vec<MoveResult>, Self::Error>;
}

impl<S: MoveScriptEngine + core::fmt::Debug> DefaultMoveEngine<S> {
    pub fn new(scripts: S) -> Self {
        Self {
            moves: Default::default(),
            scripts,
        }
    }
}

impl<S: MoveScriptEngine + core::fmt::Debug> MoveEngine for DefaultMoveEngine<S> {
    type Error = MoveError<S::Error>;

    fn execute<'a, R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        used_move: &Move,
        user: &BattlePokemon<'a>,
        target: &BattlePokemon<'a>,
    ) -> Result<Vec<MoveResult>, Self::Error> {
        match self.moves.get(&used_move.id) {
            Some(usage) => match &usage.execute {
                MoveExecution::Actions(actions) => {
                    let mut results = Vec::with_capacity(usage.execute.len());
                    user.move_actions(random, &mut results, actions, used_move, usage, target);
                    Ok(results)
                }
                MoveExecution::Script(id) => self
                    .scripts
                    .execute(random, used_move, &usage, id, user, target)
                    .map_err(MoveError::Error),
                MoveExecution::None => Ok(vec![MoveResult::NoHit(NoHitResult::Todo)]),
            },
            None => Err(MoveError::Missing),
        }
    }
}

#[derive(Debug)]
pub enum MoveError<E: Error> {
    Error(E),
    Missing,
}

impl<E: Error> Error for MoveError<E> {}

impl<E: Error> core::fmt::Display for MoveError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MoveError::Error(e) => core::fmt::Display::fmt(e, f),
            other => core::fmt::Debug::fmt(other, f),
        }
    }
}
