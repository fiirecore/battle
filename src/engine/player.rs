use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use pokedex::item::bag::OwnedBag;

use crate::{
    endpoint::{BattleEndpoint, ConnectionError},
    message::{ClientMessage, ServerMessage},
    party::PlayerParty,
    player::{PlayerSettings, RemovalReason},
    pokemon::BattlePokemon,
};

use super::active::ActiveBattlePokemon;

pub(crate) type PlayerEndpoint<ID, T> =
    Arc<dyn BattleEndpoint<ServerMessage<ID, T>, ClientMessage<ID>> + Send + Sync + 'static>;

pub struct BattlePlayer<ID, T> {
    pub party: PlayerParty<ID, ActiveBattlePokemon<ID>, BattlePokemon, T>,
    pub bag: OwnedBag,
    pub settings: PlayerSettings,
    pub endpoint: PlayerEndpoint<ID, T>,
    /// Finished the battle,
    pub(crate) removed: Option<RemovalReason>,
    /// Ready to be sent game messages
    pub(crate) ready: AtomicBool,
}

impl<ID, T> BattlePlayer<ID, T> {
    pub fn id(&self) -> &ID {
        &self.party.id
    }

    pub(crate) fn send(&self, message: ServerMessage<ID, T>) -> Result<(), ConnectionError> {
        self.endpoint.send(message)
    }

    pub fn receive(&self) -> Result<Option<ClientMessage<ID>>, ConnectionError> {
        self.endpoint.receive()
    }

    pub fn ready(&self) {
        self.ready.store(true, Ordering::Relaxed);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}
