//! Pokemon battle simulation

pub extern crate firecore_pokedex as pokedex;
#[cfg(feature = "host")]
mod host;
#[cfg(feature = "ai")]
mod ai;

mod data;
pub use data::*;

pub mod message;
pub mod moves;
pub mod party;
pub mod player;
pub mod pokemon;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Indexed<ID, T>(pokemon::PokemonIndex<ID>, T);

pub trait BattleEndpoint<ID, const AS: usize> {
    fn send(&mut self, message: message::ServerMessage<ID, AS>);

    fn receive(&mut self) -> Option<message::ClientMessage<ID>>;
}

pub mod prelude {

    #[cfg(feature = "host")]
    pub use crate::host::*;
    #[cfg(feature = "ai")]
    pub use crate::ai::*;
    #[cfg(feature = "scripting")]
    pub use crate::moves::engine::default::*;

    pub use crate::message::*;
    pub use crate::player::*;
    pub use crate::{BattleData, BattleType, BattleEndpoint};

}
