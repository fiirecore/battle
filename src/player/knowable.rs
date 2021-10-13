use core::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use pokedex::pokemon::{owned::OwnedPokemon, PokemonId, Pokemon};

use crate::{party::PlayerParty, pokemon::battle::UnknownPokemon};

pub type LocalPlayer<'d, ID> = PlayerKnowable<ID, OwnedPokemon<'d>>;

pub type UninitRemotePlayer<ID> = RemotePlayerKind<ID, PokemonId>;
pub type InitRemotePlayer<'d, ID> = RemotePlayerKind<ID, &'d Pokemon>;

pub type RemotePlayerKind<ID, P> = PlayerKnowable<ID, Remote<P>>;

type PartyKind<ID, P> = PlayerParty<ID, usize, P>;
type Remote<P> = Option<UnknownPokemon<P>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerKnowable<ID, P> {
    pub name: Option<String>,
    pub party: PartyKind<ID, P>,
}

impl<ID, P> PartyKind<ID, Remote<P>> {
    pub fn add_unknown(&mut self, index: usize, unknown: UnknownPokemon<P>) {
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
