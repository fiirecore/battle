use serde::{Deserialize, Serialize};

use pokedex::pokemon::party::Party;

use crate::pokemon::{remote::RemotePokemon, PokemonView};

pub type RemoteParty<ID, T> = crate::party::PlayerParty<ID, usize, Option<RemotePokemon>, T>;

pub type Active<A> = Vec<Option<A>>;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PlayerParty<ID, A, P, T> {
    pub id: ID,
    pub name: Option<String>,
    pub active: Active<A>,
    pub pokemon: Party<P>,
    pub trainer: Option<T>,
}

/// Get the index of the pokemon in the party from this type.
pub trait ActivePokemon: From<usize> {
    fn index(&self) -> usize;
}

impl ActivePokemon for usize {
    fn index(&self) -> usize {
        *self
    }
}

impl<ID, A, P, T> PlayerParty<ID, A, P, T> {
    pub fn id(&self) -> &ID {
        &self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unknown")
    }
}

impl<ID, A: ActivePokemon, P, T> PlayerParty<ID, A, P, T> {
    pub fn index(&self, index: usize) -> Option<usize> {
        self.active
            .get(index).and_then(|active| active.as_ref().map(ActivePokemon::index))
    }

    pub fn active(&self, active: usize) -> Option<&P> {
        self.index(active).and_then(move |index| self.pokemon.get(index))
    }

    pub fn active_mut(&mut self, active: usize) -> Option<&mut P> {
        self.index(active).and_then(move |index| self.pokemon.get_mut(index))
    }

    pub fn active_contains(&self, index: usize) -> bool {
        self.active
            .iter()
            .flatten()
            .any(|active| active.index() == index)
    }

    pub fn active_iter(&self) -> impl DoubleEndedIterator<Item = (usize, &P)> + '_ {
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

    pub fn remove_active(&mut self, active: usize) {
        if let Some(active) = self.active.get_mut(active) {
            *active = None;
        }
    }

    pub fn add(&mut self, index: usize, pokemon: P) {
        if self.pokemon.len() > index {
            self.pokemon[index] = pokemon;
        }
    }

    pub fn take(&mut self, active: usize) -> Option<P> {
        self.index(active).and_then(|index| {
                if self.pokemon.len() < index {
                    Some(self.pokemon.remove(index))
                } else {
                    None
                }
            })
    }
}

impl<ID, A: ActivePokemon, P: PokemonView, T> PlayerParty<ID, A, P, T> {
    pub fn new(
        id: ID,
        name: Option<String>,
        active: usize,
        pokemon: Party<P>,
        trainer: Option<T>,
    ) -> Self {
        let mut active = Vec::with_capacity(active);
        for (index, pokemon) in pokemon.iter().enumerate() {
            if !pokemon.fainted() {
                active.push(Some(index.into()));
            }

            if active.len() == active.capacity() {
                break;
            }
        }

        while active.capacity() != active.len() {
            active.push(None);
        }

        Self {
            id,
            name,
            active,
            pokemon,
            trainer,
        }
    }

    pub fn remaining(&self) -> impl Iterator<Item = (usize, &P)> + '_ {
        self.pokemon
            .iter()
            .enumerate()
            .filter(move |(index, p)| !self.active_contains(*index) && !p.fainted())
    }

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

    pub fn active_fainted(&self) -> Option<usize> {
        self.active_iter()
            .find(|(.., p)| p.fainted())
            .map(|(i, ..)| i)
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
