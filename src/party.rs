use serde::{Deserialize, Serialize};

use pokedex::pokemon::party::Party;

use crate::pokemon::PokemonView;

mod battle;
mod knowable;

pub use battle::*;
pub use knowable::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerParty<TEAM, A: PartyIndex, P> {
    pub id: TEAM,
    pub active: Vec<Option<A>>,
    pub pokemon: Party<P>,
}

pub trait PartyIndex: From<usize> {
    fn index(&self) -> usize;
}

impl<ID, A: PartyIndex, P> PlayerParty<ID, A, P> {
    pub fn index(&self, index: usize) -> Option<usize> {
        self.active
            .get(index)
            .map(|active| active.as_ref().map(PartyIndex::index))
            .flatten()
    }

    pub fn active(&self, active: usize) -> Option<&P> {
        self.index(active)
            .map(move |index| self.pokemon.get(index))
            .flatten()
    }

    pub fn active_mut(&mut self, active: usize) -> Option<&mut P> {
        self.index(active)
            .map(move |index| self.pokemon.get_mut(index))
            .flatten()
    }

    pub fn active_contains(&self, index: usize) -> bool {
        self.active
            .iter()
            .flatten()
            .any(|active| active.index() == index)
    }

    pub fn active_iter(&self) -> impl Iterator<Item = (usize, &P)> + '_ {
        self.active
            .iter()
            .enumerate()
            .flat_map(move |(index, active)| {
                active
                    .as_ref()
                    .map(|a| self.pokemon.get(a.index()).map(|p| (index, p)))
            })
            .flatten()
    }

    pub fn remove(&mut self, active: usize) -> Option<P> {
        self.index(active)
            .map(|index| {
                if self.pokemon.len() < index {
                    Some(self.pokemon.remove(index))
                } else {
                    None
                }
            })
            .flatten()
    }
}

impl<ID, A: PartyIndex, V: PokemonView> PlayerParty<ID, A, V> {
    pub fn all_fainted(&self) -> bool {
        !self.pokemon.iter().any(|p| !p.fainted()) || self.pokemon.is_empty()
    }

    pub fn any_inactive(&self) -> bool {
        self.pokemon
            .iter()
            .enumerate()
            .filter(|(i, ..)| !self.active_contains(*i))
            .any(|(.., pokemon)| !pokemon.fainted())
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
