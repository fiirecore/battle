// #![feature(map_into_keys_values)] // for move queue fn

pub extern crate firecore_pokedex as pokedex;

#[cfg(feature = "host")]
mod battle;
#[cfg(feature = "host")]
pub use battle::*;

mod data;
pub use data::*;

pub mod message;
pub mod moves;
pub mod party;
pub mod player;
pub mod pokemon;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct BoundAction<ID, T> {
    pub pokemon: pokemon::PokemonIndex<ID>,
    pub action: T,
}

pub trait BattleEndpoint<ID> {
    fn send(&mut self, message: message::ServerMessage<ID>);

    fn receive(&mut self) -> Option<message::ClientMessage>;
}
