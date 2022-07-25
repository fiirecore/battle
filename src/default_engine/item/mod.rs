use core::{fmt::Display, ops::Deref};
use std::error::Error;

use log::debug;
use rand::Rng;

use firecore_pokedex::{
    item::{Item, ItemId},
    moves::Move,
    pokemon::Pokemon,
};

use crate::{
    data::{BattleData, BattleType},
    engine::Players,
    item::engine::{ItemEngine, ItemResult},
    pokemon::PokemonIdentifier,
};

use super::{DefaultBattleEngine, ScriptingEngine};

mod execution;
pub use execution::*;

impl<S: ScriptingEngine> ItemEngine for DefaultBattleEngine<S> {
    type Error = ItemError<S::Error>;

    fn execute<
        ID: PartialEq,
        R: Rng,
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
        PLR: Players<ID, P, M, I>,
    >(
        &mut self,
        battle: &BattleData,
        random: &mut R,
        item: &Item,
        user: &ID,
        target: PokemonIdentifier<ID>,
        players: &mut PLR,
    ) -> Result<Vec<ItemResult>, Self::Error> {
        match self.items.get(&item.id) {
            Some(execution) => match execution {
                BattleItemExecution::Normal(..) => {
                    // to - do: fix this function
                    debug!("fix OwnedPokemon::try_use_item");
                    // match players.get_mut(&target) {
                    //     Some(pokemon) => {
                    //         pokemon.try_use_item(item);
                    //         Ok(vec![])
                    //     }
                    //     None => Err(ItemError::NoTarget),
                    // }
                    Err(ItemError::Unimplemented)
                }
                BattleItemExecution::Script => {
                    #[cfg(feature = "default_engine_scripting")]
                    return self
                        .scripting
                        .execute_item(battle, random, item, user, target, players).map_err(ItemError::Script);
                    #[cfg(not(feature = "default_engine_scripting"))]
                    return Err(ItemError::Unimplemented);
                }
                BattleItemExecution::Pokeball => match battle.type_ {
                    BattleType::Wild => Ok(match players.take(&target) {
                        Some(pokemon) => vec![ItemResult::Catch(pokemon.p.uninit())],
                        None => Vec::new(),
                    }),
                    _ => Err(ItemError::Pokeball),
                },
            },
            None => Err(ItemError::Missing(item.id)),
        }
    }
}

#[derive(Debug)]
pub enum ItemError<S: Error> {
    Script(S),
    Missing(ItemId),
    NoTarget,
    Pokeball,
    Unimplemented,
}

impl<S: Error> Error for ItemError<S> {}

impl<S: Error> Display for ItemError<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ItemError::Script(s) => Display::fmt(s, f),
            other => core::fmt::Debug::fmt(other, f),
        }
    }
}
