//! Pokemon battle simulation

pub extern crate firecore_pokedex as pokedex;

#[cfg(feature = "engine")]
pub mod engine;
#[cfg(feature = "host")]
pub mod host;

#[cfg(feature = "ai")]
pub mod ai;


pub mod data;
pub mod endpoint;
pub mod message;
pub mod moves;
pub mod party;
pub mod player;
pub mod pokemon;

pub mod prelude {

    #[cfg(feature = "ai")]
    pub use crate::ai::*;
    
    #[cfg(feature = "host")]
    pub use crate::host::prelude::*;

    #[cfg(feature = "engine")]
    pub use crate::engine::prelude::*;

    pub use crate::data::*;
    pub use crate::message::*;
    pub use crate::player::*;
}
