//! Pokemon battle simulation

extern crate alloc;

pub extern crate firecore_pokedex as pokedex;

pub mod moves;
pub mod data;
pub mod endpoint;
pub mod message;
pub mod party;
pub mod player;
pub mod pokemon;

#[cfg(feature = "host")]
pub mod host;

#[cfg(feature = "engine")]
pub mod engine;

#[cfg(feature = "default_engine")]
pub mod default_engine;

#[cfg(feature = "ai")]
pub mod ai;

pub mod prelude {

    #[cfg(feature = "ai")]
    pub use crate::ai::*;

    #[cfg(feature = "host")]
    pub use crate::host::prelude::*;

    #[cfg(feature = "default_engine")]
    pub use crate::default_engine::prelude::*;

    pub use crate::data::*;
    pub use crate::message::*;
    pub use crate::player::*;
}
