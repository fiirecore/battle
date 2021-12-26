use serde::{Deserialize, Serialize};

use pokedex::pokemon::{owned::OwnablePokemon, Health};

pub mod remote;
pub mod stat;

pub type ActivePosition = usize;
pub type PartyPosition = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PokemonIdentifier<ID>(pub ID, pub usize);

impl<ID> PokemonIdentifier<ID> {
    pub fn team(&self) -> &ID {
        &self.0
    }

    pub fn index(&self) -> usize {
        self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Indexed<ID, T>(pub PokemonIdentifier<ID>, pub T);

impl<ID: core::fmt::Display> core::fmt::Display for PokemonIdentifier<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} #{}", self.0, self.1)
    }
}

pub trait PokemonView {
    fn fainted(&self) -> bool;
}

impl<P> PokemonView for Option<remote::UnknownPokemon<P>> {
    fn fainted(&self) -> bool {
        self.as_ref().map(|u| u.fainted()).unwrap_or_default()
    }
}

impl<P, M, I, N, G> PokemonView for OwnablePokemon<P, M, I, G, N, Health> {
    fn fainted(&self) -> bool {
        OwnablePokemon::<P, M, I, G, N, Health>::fainted(self)
    }
}
