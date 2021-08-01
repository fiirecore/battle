use serde::{Deserialize, Serialize};

use pokedex::pokemon::Party;

use crate::pokemon::PokemonView;

mod knowable;
mod battle;

pub use knowable::*;
pub use battle::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerParty<ID, A: PartyIndex, P> {
    pub id: ID,
    pub active: Vec<Option<A>>,
    pub pokemon: Party<P>,
}

pub trait PartyIndex: From<usize> {
    fn index(&self) -> usize;
}

impl<ID, A: PartyIndex, P> PlayerParty<ID, A, P> {

    pub fn active(&self, active: usize) -> Option<&P> {
        self.index(active)
            .map(|index| self.pokemon.get(index))
            .flatten()
    }

    pub fn active_mut(&mut self, active: usize) -> Option<&mut P> {
        self.index(active)
            .map(move |index| self.pokemon.get_mut(index))
            .flatten()
    }

    pub fn index(&self, index: usize) -> Option<usize> {
        self.active
            .get(index)
            .map(|active| active.as_ref().map(PartyIndex::index))
            .flatten()
    }

    pub fn active_contains(&self, index: usize) -> bool {
        self.active
            .iter()
            .flatten()
            .any(|active| active.index() == index)
    }

    pub fn active_iter(&self) -> impl Iterator<Item = (usize, &P)> + '_ {
        self.active.iter().enumerate().flat_map(move |(index, active)| active.as_ref().map(|a| self.pokemon.get(a.index()).map(|p| (index, p)))).flatten()
    }

    // pub fn active_iter_mut(&mut self) -> impl Iterator<Item = &mut P> + '_ {
    //     self.active.iter().flat_map(|a| a.as_ref().map(A::index)).flat_map(move |index| self.pokemon.get_mut(index))
    // }

}

impl<ID, A: PartyIndex, P: PokemonView> PlayerParty<ID, A, P> {

    pub fn all_fainted(&self) -> bool {
        !self
            .pokemon
            .iter()
            .any(|pokemon| !pokemon.available())
            || self.pokemon.is_empty()
    }

    pub fn any_inactive(&self) -> bool {
        self.pokemon
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.active_contains(*i))
            .any(|(_, pokemon)| !pokemon.available())
    }

    pub fn needs_replace(&self) -> bool {
        self.any_inactive() && self.active.iter().any(Option::is_none)
    }

    pub fn replace(&mut self, active: usize, new: Option<usize>) {
        if let Some(a) = self.active.get_mut(active) {
            *a = new.map(Into::into);
        }
    }

}

impl<ID: Default, A: PartyIndex, P> Default for PlayerParty<ID, A, P> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            active: Default::default(),
            pokemon: Default::default(),
        }
    }
}

impl PartyIndex for usize {
    fn index(&self) -> usize {
        *self
    }
}