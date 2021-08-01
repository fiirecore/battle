use core::ops::Deref;

use pokedex::pokemon::{Party, PokemonInstance, PokemonParty};

use crate::{
    party::PlayerParty,
    pokemon::{ActivePokemon, BattlePokemon, UnknownPokemon},
};

pub type BattleParty<ID> = PlayerParty<ID, ActivePokemon, BattlePokemon>;

impl<ID> BattleParty<ID> {
    pub fn all_fainted(&self) -> bool {
        !self
            .pokemon
            .iter()
            .any(|pokemon| !pokemon.fainted() && !pokemon.caught)
            || self.pokemon.is_empty()
    }

    pub fn any_inactive(&self) -> bool {
        self.pokemon
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.active_contains(*i))
            .any(|(_, pokemon)| !pokemon.fainted() && !pokemon.caught)
    }

    pub fn know(&mut self, index: usize) -> Option<UnknownPokemon> {
        self.pokemon
            .get_mut(index)
            .map(BattlePokemon::know)
            .flatten()
    }

    pub fn needs_replace(&self) -> bool {
        self.any_inactive() && self.active.iter().any(Option::is_none)
    }

    pub fn reveal_active(&mut self) {
        for index in self.active.iter().flatten().map(|a| a.index) {
            if let Some(pokemon) = self.pokemon.get_mut(index) {
                pokemon.known = true;
            }
        }
    }

    pub fn replace(&mut self, active: usize, new: Option<usize>) {
        if let Some(a) = self.active.get_mut(active) {
            *a = new.map(ActivePokemon::new);
        }
    }

    pub fn ready_to_move(&self) -> bool {
        self.active
            .iter()
            .flatten()
            .all(|a| a.queued_move.is_some())
    }
}

impl<ID> BattleParty<ID> {
    pub fn party_ref(&self) -> Party<&PokemonInstance> {
        self.pokemon.iter().map(Deref::deref).collect()
    }
}

impl<ID> BattleParty<ID> {
    pub fn party_cloned(&self) -> PokemonParty {
        self.pokemon.iter().map(|b| b.deref().clone()).collect()
    }
}
