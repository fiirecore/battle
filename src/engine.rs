use core::ops::Deref;

use rand::Rng;

use firecore_pokedex::{item::Item, moves::Move, pokemon::Pokemon};

use crate::pokemon::PokemonIdentifier;

pub mod pokemon;
pub use pokemon::BattlePokemon;

pub trait Players<
    ID: PartialEq,
    P: Deref<Target = Pokemon>,
    M: Deref<Target = Move>,
    I: Deref<Target = Item>,
>
{
    fn create_targets<R: Rng>(
        &self,
        user: &PokemonIdentifier<ID>,
        m: &Move,
        targeting: Option<PokemonIdentifier<ID>>,
        random: &mut R,
    ) -> Vec<PokemonIdentifier<ID>>;

    fn get(&self, id: &PokemonIdentifier<ID>) -> Option<&BattlePokemon<P, M, I>>;

    fn get_mut(&mut self, id: &PokemonIdentifier<ID>) -> Option<&mut BattlePokemon<P, M, I>>;

    fn take(&mut self, id: &PokemonIdentifier<ID>) -> Option<BattlePokemon<P, M, I>>;
}
