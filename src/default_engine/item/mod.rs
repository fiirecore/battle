use core::{fmt::Display, ops::Deref};
use std::error::Error;

use log::debug;
use rand::Rng;

use firecore_pokedex::{
    item::{Item, ItemId},
    moves::Move,
    pokemon::Pokemon,
    Uninitializable,
};

use crate::{
    data::{BattleData, BattleType},
    engine::Players,
    item::engine::{ItemEngine, ItemResult},
    pokemon::PokemonIdentifier,
};

use super::{DefaultEngine, ScriptError};

#[cfg(feature = "default_engine_scripting")]
pub mod scripting;

mod execution;
pub use execution::*;

impl ItemEngine for DefaultEngine {
    type Error = ItemError;

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
    ) -> Result<Vec<ItemResult>, Self::Error> {
        match self.items.get(&item.id) {
            Some(execution) => match execution {
                BattleItemExecution::Normal(..) => {
                    // to - do: fix this function
                    debug!("fix OwnedPokemon::try_use_item");
                    match players.get_mut(&target) {
                        Some(pokemon) => {
                            pokemon.try_use_item(item);
                            Ok(vec![])
                        }
                        None => Err(ItemError::NoTarget),
                    }
                }
                BattleItemExecution::Script => {
                    #[cfg(feature = "default_engine_scripting")]
                    return self
                        .scripting
                        .execute_item(battle, random, item, user, target, players);
                    #[cfg(not(feature = "default_engine_scripting"))]
                    return Err(ItemError::Script(ScriptError::default()));
                }
                BattleItemExecution::Pokeball => match battle.type_ {
                    BattleType::Wild => Ok(match players.take(&target) {
                        Some(pokemon) => vec![ItemResult::Catch(pokemon.p.uninit())],
                        None => Vec::new(),
                    }),
                    _ => Err(ItemError::TrainerBattlePokeball),
                },
            },
            None => Err(ItemError::Missing(item.id)),
        }
    }
}

#[derive(Debug)]
pub enum ItemError {
    Script(ScriptError),
    Missing(ItemId),
    NoTarget,
    TrainerBattlePokeball,
}

impl Error for ItemError {}

impl Display for ItemError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ItemError::Script(s) => Display::fmt(s, f),
            other => core::fmt::Debug::fmt(other, f),
        }
    }
}
