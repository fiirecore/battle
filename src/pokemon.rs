use std::ops::Deref;

use serde::{Deserialize, Serialize};

use pokedex::{
    item::Item,
    moves::Move,
    pokemon::{owned::OwnedPokemon, Pokemon},
};

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
    // fn id(&self) -> &PokemonId;

    fn fainted(&self) -> bool;
}

impl<P> PokemonView for Option<remote::UnknownPokemon<P>> {
    // fn id(&self) -> &PokemonId {
    //     self.as_ref()
    //         .map(|u| u.pokemon.as_id())
    //         .unwrap_or(&Pokemon::UNKNOWN)
    // }

    fn fainted(&self) -> bool {
        self.as_ref().map(|u| u.fainted()).unwrap_or_default()
    }
}

impl<P> PokemonView for remote::UnknownPokemon<P> {
    // fn id(&self) -> &PokemonId {
    //     self.pokemon.as_id()
    // }

    fn fainted(&self) -> bool {
        remote::UnknownPokemon::fainted(self)
    }
}

impl<
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
    > PokemonView for OwnedPokemon<P, M, I>
{
    fn fainted(&self) -> bool {
        OwnedPokemon::<P, M, I>::fainted(self)
    }

    // fn id(&self) -> &PokemonId {
    //     &self.pokemon.id
    // }
}
