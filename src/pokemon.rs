use core::fmt::Display;
use serde::{Deserialize, Serialize};

mod active;
pub use active::*;

mod view;
pub use view::*;

mod unknown;
pub use unknown::*;

mod battle;
pub use battle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PokemonIndex<ID> {
    pub team: ID,
    pub index: usize,
}

impl<ID: Sized + Display> Display for PokemonIndex<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} #{}", self.team, self.index)
    }
}
