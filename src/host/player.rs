use core::cell::Ref;
use rand::Rng;

use pokedex::{
    item::Item,
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

pub type BattlePlayer<'d, ID> =
    Player<ID, ActiveBattlePokemon<ID>, HostPokemon<'d>, Box<dyn BattleEndpoint<ID>>>;

pub struct PlayerData<ID> {
    pub id: ID,
    pub name: Option<String>,
    pub party: Party<SavedPokemon>,
    pub settings: PlayerSettings,
    pub endpoint: Box<dyn BattleEndpoint<ID>>,
}

impl<'d, ID> BattlePlayer<'d, ID> {
    pub fn send(&mut self, message: ServerMessage<ID>) {
        self.endpoint.send(message)
    }

    pub fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>> {
        self.endpoint.receive()
    }
}

impl<ID> PlayerData<ID> {
    pub(crate) fn init<'d>(
        self,
        random: &mut impl Rng,
        active: usize,
        pokedex: &'d dyn Dex<'d, Pokemon, &'d Pokemon>,
        movedex: &'d dyn Dex<'d, Move, &'d Move>,
        itemdex: &'d dyn Dex<'d, Item, &'d Item>,
    ) -> BattlePlayer<'d, ID> {
        let pokemon: Party<HostPokemon<'d>> = self
            .party
            .into_iter()
            .flat_map(|p| p.init(random, pokedex, movedex, itemdex))
            .map(Into::into)
            .collect();

        let mut party = PlayerParty::new(self.id, self.name, active, pokemon);

        for index in party.active.iter().flatten().map(ActivePokemon::index) {
            if let Some(pokemon) = party.pokemon.get_mut(index) {
                pokemon.known = true;
            }
        }

        BattlePlayer {
            party,
            settings: self.settings,
            endpoint: self.endpoint,
        }
    }
}

impl<ID: Clone> ClientPlayerData<ID> {
    pub fn new<'d: 'a, 'a, I: Iterator<Item = Ref<'a, BattlePlayer<'d, ID>>> + 'a>(
        data: BattleData,
        player: &BattlePlayer<ID>,
        others: I,
    ) -> Self
    where
        ID: 'a,
    {
        Self {
            id: player.party.id().clone(),
            name: player.party.name.clone(),
            active: ActiveBattlePokemon::as_usize(&player.party.active),
            data,
            remotes: others.map(|player| player.party.as_remote()).collect(),
        }
    }
}
