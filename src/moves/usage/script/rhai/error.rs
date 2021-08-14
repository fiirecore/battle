use std::{error::Error, fmt::{Display, Debug, Formatter, Result as FmtResult}};

use rhai::EvalAltResult;

#[derive(Debug)]
pub enum RhaiMoveError {
    Rhai(Box<EvalAltResult>),
    Missing,
}

impl Error for RhaiMoveError {}

impl Display for RhaiMoveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RhaiMoveError::Rhai(err) => Display::fmt(err, f),
            _ => Debug::fmt(self, f),
        }
    }
}

impl From<Box<EvalAltResult>> for RhaiMoveError {
    fn from(r: Box<EvalAltResult>) -> Self {
        Self::Rhai(r)
    }
}