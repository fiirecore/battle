use pokedex::Uninitializable;

use crate::{
    party::{PartyIndex, PlayerParty, RemoteParty},
    pokemon::{
        remote::{RemotePokemon, UnknownPokemon},
    },
};

use super::pokemon::{ActivePokemon, BattlePokemon};

pub type BattleParty<'d, ID, const AS: usize> =
    PlayerParty<ID, ActivePokemon<ID>, BattlePokemon<'d>, AS>;

impl<'d, ID, const AS: usize> BattleParty<'d, ID, AS> {
    pub fn know(&mut self, index: usize) -> Option<RemotePokemon> {
        self.pokemon
            .get_mut(index)
            .map(BattlePokemon::know)
            .flatten()
    }

    pub fn reveal_active(&mut self) {
        for index in self.active.iter().flatten().map(PartyIndex::index) {
            if let Some(pokemon) = self.pokemon.get_mut(index) {
                pokemon.known = true;
            }
        }
    }

    pub fn ready_to_move(&self) -> bool {
        self.active
            .iter()
            .flatten()
            .all(ActivePokemon::queued) || self.active.iter().all(Option::is_none)
    }

    pub fn as_remote(&self) -> RemoteParty<ID, AS>
    where
        ID: Clone,
    {
        RemoteParty {
            id: self.id.clone(),
            name: self.name.clone(),
            pokemon: self
                .pokemon
                .iter()
                .map(|p| p.known.then(|| UnknownPokemon::new(p).uninit()))
                .collect(),
            active: ActivePokemon::into_remote(&self.active),
        }
    }
}
