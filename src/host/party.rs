use core::ops::Deref;

use pokedex::{item::Item, moves::Move, pokemon::Pokemon};

use crate::{
    party::{PlayerParty, RemoteParty},
    pokemon::remote::{RemotePokemon, UnknownPokemon},
};

use super::pokemon::{ActiveBattlePokemon, HostPokemon};

pub type BattleParty<ID, P, M, I> = PlayerParty<ID, ActiveBattlePokemon<ID>, HostPokemon<P, M, I>>;

impl<ID, P: Deref<Target = Pokemon> + Clone, M: Deref<Target = Move>, I: Deref<Target = Item>>
    BattleParty<ID, P, M, I>
{
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

    pub fn as_remote(&self) -> RemoteParty<ID>
    where
        ID: Clone,
        P: Clone,
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
