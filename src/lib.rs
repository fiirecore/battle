//! Pokemon battle simulation

pub extern crate firecore_pokedex as pokedex;
#[cfg(feature = "host")]
pub mod host;
#[cfg(feature = "ai")]
pub mod ai;

mod data;
pub use data::*;

pub mod message;
pub mod moves;
pub mod party;
pub mod player;
pub mod pokemon;
pub mod endpoint;

pub mod prelude {

    #[cfg(feature = "host")]
    pub use crate::host::prelude::*;
    #[cfg(feature = "ai")]
    pub use crate::ai::BattleAi;

    pub use crate::message::*;
    pub use crate::player::*;
    pub use crate::{BattleData, BattleType};

}
