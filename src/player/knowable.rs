use serde::{Deserialize, Serialize};
use core::ops::{Deref, DerefMut};

use pokedex::pokemon::PokemonInstance;

use crate::{pokemon::UnknownPokemon, party::PlayerParty};

pub type LocalPlayer<ID> = PlayerKnowable<ID, PokemonInstance>;
pub type RemotePlayer<ID> = PlayerKnowable<ID, Option<UnknownPokemon>>;

type PartyKind<ID, P> = PlayerParty<ID, usize, P>;

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerKnowable<ID, P> {
    pub name: Option<String>,
    pub party: PartyKind<ID, P>,
}

impl<ID: Default, P> Default for PlayerKnowable<ID, P> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            party: Default::default(),
        }
    }
}

impl<ID> PartyKind<ID, Option<UnknownPokemon>> {
    pub fn add_instance(&mut self, index: usize, instance: PokemonInstance) {
        if let Some(pokemon) = self.pokemon.get_mut(index) {
            let pokemon = pokemon.get_or_insert(UnknownPokemon::new(&instance));
            pokemon.instance = Some(instance);
        }
    }

    pub fn add_unknown(&mut self, index: usize, unknown: UnknownPokemon) {
        if self.pokemon.len() > index {
            self.pokemon[index] = Some(unknown);
        }
    }
}

impl<ID, P> Deref for PlayerKnowable<ID, P> {
    type Target = PartyKind<ID, P>;

    fn deref(&self) -> &Self::Target {
        &self.party
    }
}

impl<ID, P> DerefMut for PlayerKnowable<ID, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.party
    }
}