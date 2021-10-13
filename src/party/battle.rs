use core::ops::Deref;

use pokedex::pokemon::{owned::OwnedPokemon, party::Party};

use crate::{
    party::{PartyIndex, PlayerParty},
    pokemon::{
        battle::{BattlePokemon, InitUnknownPokemon},
        ActivePokemon,
    },
};

pub type BattleParty<'d, TEAM> = PlayerParty<TEAM, ActivePokemon, BattlePokemon<'d>>;

impl<'d, TEAM> BattleParty<'d, TEAM> {
    pub fn know(&mut self, index: usize) -> Option<InitUnknownPokemon<'d>> {
        self.pokemon
            .get_mut(index)
            .map(BattlePokemon::know)
            .flatten()
    }

    pub fn reveal_active(&mut self) {
        for index in self.active.iter().flatten().map(PartyIndex::index) {
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

impl<'d, TEAM> BattleParty<'d, TEAM> {
    pub fn party_ref(&self) -> Party<&OwnedPokemon<'d>> {
        self.pokemon.iter().map(Deref::deref).collect()
    }

    pub fn party_cloned(&self) -> Party<OwnedPokemon<'d>> {
        self.pokemon.iter().map(Deref::deref).cloned().collect()
    }
}
