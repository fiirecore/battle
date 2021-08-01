use core::ops::Deref;

use pokedex::pokemon::{Party, PokemonInstance, PokemonParty};

use crate::{
    party::PlayerParty,
    pokemon::{ActivePokemon, BattlePokemon, UnknownPokemon},
};

pub type BattleParty<ID> = PlayerParty<ID, ActivePokemon, BattlePokemon>;

impl<ID> BattleParty<ID> {

    pub fn know(&mut self, index: usize) -> Option<UnknownPokemon> {
        self.pokemon
            .get_mut(index)
            .map(BattlePokemon::know)
            .flatten()
    }

    pub fn reveal_active(&mut self) {
        for index in self.active.iter().flatten().map(|a| a.index) {
            if let Some(pokemon) = self.pokemon.get_mut(index) {
                pokemon.known = true;
            }
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
