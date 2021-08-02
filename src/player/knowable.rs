use core::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use pokedex::pokemon::PokemonInstance;

use crate::{party::PlayerParty, pokemon::UnknownPokemon};

pub type LocalPlayer<ID> = PlayerKnowable<ID, PokemonInstance>;
pub type RemotePlayer<ID> = PlayerKnowable<ID, Remote>;

type PartyKind<ID, P> = PlayerParty<ID, usize, P>;
type Remote = Option<UnknownPokemon>;

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerKnowable<ID, P> {
    pub name: Option<String>,
    pub party: PartyKind<ID, P>,
}

impl<ID> PartyKind<ID, Remote> {
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

impl<ID: Default, P> Default for PlayerKnowable<ID, P> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            party: Default::default(),
        }
    }
}
