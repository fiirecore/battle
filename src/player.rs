use pokedex::{
    pokemon::{owned::OwnedPokemon, party::Party},
    Uninitializable,
};

use crate::{BattleEndpoint, message::{ClientMessage, ServerMessage}, party::{BattleParty, PlayerParty}, player::UninitRemotePlayer, pokemon::battle::{BattlePokemon, UnknownPokemon}};

mod settings;
pub use settings::*;

mod knowable;
pub use knowable::*;

mod validate;
pub use validate::*;

#[cfg(feature = "ai")]
pub mod ai;

pub struct BattlePlayer<'d, ID, E> {
    name: Option<String>,
    pub party: BattleParty<'d, ID>,
    endpoint: E,
    pub settings: PlayerSettings,
    /// Player's turn has finished
    pub waiting: bool,
}

impl<'d, ID, E: BattleEndpoint<ID>> BattlePlayer<'d, ID, E> {
    pub fn new(
        id: ID,
        party: Party<OwnedPokemon<'d>>,
        name: Option<String>,
        settings: PlayerSettings,
        endpoint: E,
        active_size: usize,
    ) -> Self {
        let mut active = Vec::with_capacity(active_size);
        let mut count = 0;

        while active.len() < active_size {
            match party.get(count) {
                Some(p) => {
                    if !p.fainted() {
                        active.push(Some(count.into()));
                    }
                }
                None => active.push(None),
            }
            count += 1;
        }

        Self {
            endpoint,
            party: BattleParty {
                id,
                active,
                pokemon: party.into_iter().map(BattlePokemon::from).collect(),
            },
            name,
            settings,
            waiting: false,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unknown")
    }

    pub fn send(&mut self, message: ServerMessage<ID>) {
        self.endpoint.send(message)
    }

    pub fn receive(&mut self) -> Option<ClientMessage> {
        self.endpoint.receive()
    }

}

impl<'d, ID: core::fmt::Debug, E: BattleEndpoint<ID>> core::fmt::Debug for BattlePlayer<'d, ID, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BattlePlayer").field("party", &self.party).field("name", &self.name).finish()
    }
}

impl<'d, ID: Copy, E: BattleEndpoint<ID>> BattlePlayer<'d, ID, E> {
    pub fn as_remote(&self) -> UninitRemotePlayer<ID> {
        PlayerKnowable {
            name: self.name.clone(),
            party: PlayerParty {
                id: self.party.id,
                pokemon: self
                    .party
                    .pokemon
                    .iter()
                    .map(|p| p.known.then(|| UnknownPokemon::new(p).uninit()))
                    .collect(),
                active: self
                    .party
                    .active
                    .iter()
                    .map(|active| active.as_ref().map(|a| a.index))
                    .collect(),
            },
        }
    }
}
