use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::error::Error;

use pokedex::item::ItemId;

mod execution;
pub use execution::*;

#[derive(Debug)]
pub enum ItemError<S: Error> {
    Script(S),
    Missing(ItemId),
    NoTarget,
    Pokeball,
    Unimplemented,
}

impl<S: Error> Error for ItemError<S> {}

impl<S: Error> Display for ItemError<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ItemError::Script(s) => Display::fmt(s, f),
            other => Debug::fmt(other, f),
        }
    }
}
