use std::sync::Arc;

use pokedex::{
    item::bag::OwnedBag,
    pokemon::{owned::OwnedPokemon, party::Party},
};

use crate::{
    endpoint::{BattleEndpoint, ConnectionError},
    engine::BattlePokemon,
    message::{ClientMessage, ServerMessage},
    party::{ActivePokemon, PlayerParty},
    player::{PlayerSettings, RemovalReason},
};

use super::pokemon::ActiveBattlePokemon;

type PlayerEndpoint<ID, T> =
    Arc<dyn BattleEndpoint<ServerMessage<ID, T>, ClientMessage<ID>> + Send + Sync + 'static>;

pub struct BattlePlayer<ID, T> {
    pub party: PlayerParty<ID, ActiveBattlePokemon<ID>, BattlePokemon, T>,
    pub bag: OwnedBag,
    pub settings: PlayerSettings,
    pub endpoint: PlayerEndpoint<ID, T>,
    pub(crate) removed: Option<RemovalReason>,
}

pub struct PlayerData<ID, T> {
    pub id: ID,
    pub name: Option<String>,
    pub party: Party<OwnedPokemon>,
    pub bag: OwnedBag,
    pub trainer: Option<T>,
    pub settings: PlayerSettings,
    pub endpoint: PlayerEndpoint<ID, T>,
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
}

impl<ID, T> PlayerData<ID, T> {
    pub(crate) fn init(self, active: usize) -> BattlePlayer<ID, T> {
        let pokemon: Party<BattlePokemon> = self.party.into_iter().map(Into::into).collect();

        let mut party = PlayerParty::new(self.id, self.name, active, pokemon, self.trainer);

        for index in party.active.iter().flatten().map(ActivePokemon::index) {
            if let Some(pokemon) = party.pokemon.get_mut(index) {
                pokemon.reveal();
            }
        }

        BattlePlayer {
            party,
            bag: self.bag,
            settings: self.settings,
            endpoint: self.endpoint,
            removed: None,
        }
    }
}

// impl<ID: Clone, T: Clone> ClientPlayerData<ID, T> {
//     pub fn new<'a, ITER: Iterator<Item = Ref<'a, BattlePlayer<ID, T>>>>(
//         data: BattleData,
//         player: &BattlePlayer<ID, T>,
//         others: ITER,
//     ) -> Self
//     where
//         ID: 'a,
//         T: 'a,
//     {
//         Self {
//             local: PlayerParty {
//                 id: player.party.id().clone(),
//                 name: player.party.name.clone(),
//                 active: ActiveBattlePokemon::as_usize(&player.party.active),
//                 pokemon: player
//                     .party
//                     .pokemon
//                     .iter()
//                     .map(|p| &p.p.p)
//                     .cloned()
//                     .map(|pokemon| pokemon.uninit())
//                     .collect(),
//                 trainer: player.party.trainer.clone(),
//             },
//             data,
//             remotes: others.map(|player| player.party.as_remote()).collect(),
//             bag: player.bag.clone().uninit(),
//         }
//     }
// }
