use core::cell::Ref;
use rand::Rng;

use pokedex::{
    item::Item,
    moves::Move,
    pokemon::{party::Party, Pokemon, owned::SavedPokemon},
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

pub type BattlePlayer<'d, ID, const AS: usize> =
    Player<ID, ActiveBattlePokemon<ID>, HostPokemon<'d>, Box<dyn BattleEndpoint<ID, AS>>, AS>;

pub struct PlayerData<ID, const AS: usize> {
    pub id: ID,
    pub name: Option<String>,
    pub party: Party<SavedPokemon>,
    pub settings: PlayerSettings,
    pub endpoint: Box<dyn BattleEndpoint<ID, AS>>,
}

impl<'d, ID, const AS: usize> BattlePlayer<'d, ID, AS> {
    pub fn send(&mut self, message: ServerMessage<ID, AS>) {
        self.endpoint.send(message)
    }

    pub fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>> {
        self.endpoint.receive()
    }
}

impl<ID, const AS: usize> PlayerData<ID, AS> {
    pub(crate) fn init<'d>(
        self,
        random: &mut impl Rng,
        pokedex: &'d dyn Dex<Pokemon>,
        movedex: &'d dyn Dex<Move>,
        itemdex: &'d dyn Dex<Item>,
    ) -> BattlePlayer<'d, ID, AS> {
        let pokemon: Party<HostPokemon<'d>> = self
            .party
            .into_iter()
            .flat_map(|p| p.init(random, pokedex, movedex, itemdex))
            .map(Into::into)
            .collect();

        let mut party = PlayerParty::new(self.id, self.name, pokemon);

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

impl<ID: Clone, const AS: usize> ClientPlayerData<ID, AS> {
    pub fn new<'d: 'a, 'a, I: Iterator<Item = Ref<'a, BattlePlayer<'d, ID, AS>>> + 'a>(
        data: BattleData,
        player: &BattlePlayer<ID, AS>,
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
