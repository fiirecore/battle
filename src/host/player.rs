use core::{cell::Ref};
use rand::Rng;

use pokedex::{
    item::{bag::SavedBag, Item},
    moves::Move,
    pokemon::{owned::SavedPokemon, party::Party, Pokemon},
    Dex,
};

use crate::{
    data::BattleData,
    endpoint::{BattleEndpoint, ReceiveError},
    message::{ClientMessage, ServerMessage},
    party::{ActivePokemon, PlayerParty},
    player::{ClientPlayerData, Player, PlayerSettings},
};

use super::pokemon::{ActiveBattlePokemon, HostPokemon};

pub type BattlePlayer<ID, T> =
    Player<ID, ActiveBattlePokemon<ID>, HostPokemon, T, Box<dyn BattleEndpoint<ID, T>>>;

pub struct PlayerData<ID, T> {
    pub id: ID,
    pub name: Option<String>,
    pub party: Party<SavedPokemon>,
    pub bag: SavedBag,
    pub trainer: Option<T>,
    pub settings: PlayerSettings,
    pub endpoint: Box<dyn BattleEndpoint<ID, T>>,
}

impl<ID, T> BattlePlayer<ID, T> {
    pub fn send(&mut self, message: ServerMessage<ID, T>) {
        self.endpoint.send(message)
    }

    pub fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>> {
        self.endpoint.receive()
    }
}

impl<ID, T> PlayerData<ID, T> {
    pub(crate) fn init<R: Rng>(
        self,
        random: &mut R,
        active: usize,
        pokedex: &Dex<Pokemon>,
        movedex: &Dex<Move>,
        itemdex: &Dex<Item>,
    ) -> BattlePlayer<ID, T> {
        let pokemon: Party<HostPokemon> = self
            .party
            .into_iter()
            .flat_map(|p| p.init(random, pokedex, movedex, itemdex))
            .map(Into::into)
            .collect();

        let mut party = PlayerParty::new(self.id, self.name, active, pokemon, self.trainer);

        for index in party.active.iter().flatten().map(ActivePokemon::index) {
            if let Some(pokemon) = party.pokemon.get_mut(index) {
                pokemon.known = true;
            }
        }

        let bag = self.bag.init(itemdex).unwrap_or_default();

        BattlePlayer {
            party,
            bag,
            settings: self.settings,
            endpoint: self.endpoint,
        }
    }
}

impl<ID: Clone, T: Clone> ClientPlayerData<ID, T> {
    pub fn new<'a, ITER: Iterator<Item = Ref<'a, BattlePlayer<ID, T>>>>(
        data: BattleData,
        player: &BattlePlayer<ID, T>,
        others: ITER,
    ) -> Self
    where
        ID: 'a,
        T: 'a,
    {
        Self {
            local: PlayerParty {
                id: player.party.id().clone(),
                name: player.party.name.clone(),
                active: ActiveBattlePokemon::as_usize(&player.party.active),
                pokemon: player
                    .party
                    .pokemon
                    .iter()
                    .map(|p| &p.p.p)
                    .cloned()
                    .map(|pokemon| pokemon.uninit())
                    .collect(),
                trainer: player.party.trainer.clone(),
            },
            data,
            remotes: others.map(|player| player.party.as_remote()).collect(),
            bag: player.bag.clone().uninit(),
        }
    }
}
