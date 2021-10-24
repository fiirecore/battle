use pokedex::Uninitializable;

use crate::{
    party::{PlayerParty, RemoteParty},
    pokemon::remote::{RemotePokemon, UnknownPokemon},
};

use super::pokemon::{ActiveBattlePokemon, HostPokemon};

pub type BattleParty<'d, ID, const AS: usize> =
    PlayerParty<ID, ActiveBattlePokemon<ID>, HostPokemon<'d>, AS>;

impl<'d, ID, const AS: usize> BattleParty<'d, ID, AS> {
    pub fn know(&mut self, index: usize) -> Option<RemotePokemon> {
        self.pokemon.get_mut(index).map(HostPokemon::know).flatten()
    }

    pub fn ready_to_move(&self) -> bool {
        self.active
            .iter()
            .flatten()
            .all(ActiveBattlePokemon::queued)
            || self.active.iter().all(Option::is_none)
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
            active: ActiveBattlePokemon::as_usize(&self.active),
        }
    }
}
