use serde::{Deserialize, Serialize};

use pokedex::pokemon::owned::OwnedPokemon;

use crate::{moves::BattleMove, party::PartyIndex};

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

impl<'d> PokemonView for OwnedPokemon<'d> {
    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}

// remove Serialize + Deserialize when serde supports const generics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ActivePokemon<ID> {
    pub index: usize,
    pub queued_move: Option<BattleMove<ID>>,
}

impl<ID> ActivePokemon<ID> {

    pub fn into_remote<const AS: usize>(this: &[Option<Self>; AS]) -> [Option<usize>; AS] {
        let mut active = [None; AS];

        for (i, a) in this.iter().enumerate() {
            active[i] = a.as_ref().map(PartyIndex::index);
        }

        return active;
    }

}

impl<ID> PartyIndex for ActivePokemon<ID> {
    fn index(&self) -> usize {
        self.index
    }
}

impl<ID> From<usize> for ActivePokemon<ID> {
    fn from(index: usize) -> Self {
        Self {
            index,
            queued_move: None,
        }
    }
}
