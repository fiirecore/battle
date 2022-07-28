use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::error::Error;

use pokedex::moves::MoveId;

mod execution;
pub use execution::*;

#[derive(Debug)]
pub enum MoveError<S: Error = NoScriptError> {
    Script(S),
    Missing(MoveId),
    NoTarget,
    Unimplemented,
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
