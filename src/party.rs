use serde::{Deserialize, Serialize};

use pokedex::pokemon::Party;

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

pub trait PartyIndex {
    fn index(&self) -> usize;
}

impl<ID, A: PartyIndex, P> PlayerParty<ID, A, P> {

    pub fn active(&self, active: usize) -> Option<&P> {
        self.party_index(active)
            .map(|index| self.pokemon.get(index))
            .flatten()
    }

    pub fn active_mut(&mut self, active: usize) -> Option<&mut P> {
        self.party_index(active)
            .map(move |index| self.pokemon.get_mut(index))
            .flatten()
    }

    pub fn party_index(&self, index: usize) -> Option<usize> {
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