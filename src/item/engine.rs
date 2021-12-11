use core::ops::Deref;

use std::error::Error;

use rand::Rng;

use pokedex::{
    item::Item,
    moves::Move,
    pokemon::{owned::SavedPokemon, Pokemon},
};

use crate::{data::BattleData, engine::Players, pokemon::PokemonIdentifier};

pub trait ItemEngine {
    type Error: Error;

    fn execute<
        ID: PartialEq,
        R: Rng,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
        PLR: Players<ID, P, M, I>,
    >(
        &mut self,
        battle: &BattleData,
        random: &mut R,
        item: &Item,
        user: &ID,
        target: PokemonIdentifier<ID>,
        players: &mut PLR,
    ) -> Result<Vec<ItemResult>, Self::Error>;
}

pub enum ItemResult {
    Catch(SavedPokemon),
}
