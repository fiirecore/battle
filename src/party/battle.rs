use core::ops::Deref;

use pokedex::pokemon::{Party, PokemonRef};

use crate::{
    party::{PartyIndex, PlayerParty},
    pokemon::{battle::BattlePokemon, ActivePokemon, OwnedRefPokemon, UnknownPokemon},
};

pub type BattleParty<'d, ID> = PlayerParty<ID, ActivePokemon, BattlePokemon<'d>>;

impl<'d, ID> BattleParty<'d, ID> {
    pub fn know(&mut self, index: usize) -> Option<UnknownPokemon<PokemonRef<'d>>> {
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

impl<'d, ID> BattleParty<'d, ID> {
    pub fn party_ref(&self) -> Party<&OwnedRefPokemon<'d>> {
        self.pokemon.iter().map(Deref::deref).collect()
    }

    pub fn party_cloned(&self) -> Party<OwnedRefPokemon<'d>> {
        self.pokemon.iter().map(Deref::deref).cloned().collect()
    }
}
