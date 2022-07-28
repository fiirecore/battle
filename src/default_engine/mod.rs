use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash};

use hashbrown::HashMap;
use rand::Rng;

use pokedex::{
    item::{Item, ItemId},
    moves::{Move, MoveId},
};

use crate::{
    data::{BattleData, BattleType},
    engine::{pokemon::throw_move, BattleEngine, BattlePokemon, MoveResult, Players, ItemResult},
    pokemon::{Indexed, PokemonIdentifier},
};

pub mod item;
use self::item::*;

pub mod moves;
use self::moves::*;

pub mod scripting;
use self::scripting::ScriptingEngine;

#[cfg(feature = "default_engine_scripting")]
pub mod default_scripting;

pub(crate) mod prelude {

    pub use super::{DefaultEngine, EngineItems, EngineMoves};

    #[cfg(feature = "default_engine_scripting")]
    pub use super::scripting::*;
}

pub type EngineItems = HashMap<ItemId, BattleItemExecution>;
pub type EngineMoves = HashMap<MoveId, MoveExecution>;

#[cfg(feature = "default_engine_scripting")]
pub type DefaultEngine = DefaultBattleEngine<default_scripting::RhaiScriptingEngine>;

#[cfg(not(feature = "default_engine_scripting"))]
pub type DefaultEngine = DefaultBattleEngine<scripting::DefaultScriptEngine>;

pub struct DefaultBattleEngine<S: ScriptingEngine> {
    pub items: EngineItems,
    pub moves: EngineMoves,
    pub scripting: S,
}

#[cfg(feature = "default_engine_scripting")]
impl DefaultBattleEngine<default_scripting::RhaiScriptingEngine> {
    pub fn new<ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        Self {
            items: Default::default(),
            moves: Default::default(),
            scripting: default_scripting::RhaiScriptingEngine::new::<ID, R>(),
        }
    }
}

impl<S: ScriptingEngine> BattleEngine for DefaultBattleEngine<S> {
    type MoveError = MoveError<S::Error>;

    type ItemError = ItemError<S::Error>;

    fn execute_move<
        ID: Clone + Hash + Eq + 'static + Debug,
        R: Rng + Clone + 'static,
        PLR: Players<ID>,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::MoveError> {
        match self.moves.get(&m.id) {
            Some(usage) => {
                let targets = players.create_targets(&user.0, m, targeting, random);

                match &usage {
                    MoveExecution::Actions(actions) => {
                        let mut results = Vec::new();
                        for target_id in targets {
                            match players.get(&target_id) {
                                Some(target) => match throw_move(random, m.accuracy) {
                                    true => {
                                        results.reserve(usage.size());
                                        move_usage(
                                            &user,
                                            random,
                                            &mut results,
                                            actions,
                                            m,
                                            Indexed(target_id, target),
                                        );
                                    }
                                    false => {
                                        results.push(Indexed(user.0.clone(), MoveResult::Miss))
                                    }
                                },
                                None => (),
                            }
                        }
                        Ok(results)
                    }
                    MoveExecution::Script => {
                        #[cfg(feature = "default_engine_scripting")]
                        return self
                            .scripting
                            .execute_move(random, m, user, targets, players)
                            .map_err(MoveError::Script);
                        #[cfg(not(feature = "default_engine_scripting"))]
                        return Err(MoveError::Unimplemented);
                    }
                    MoveExecution::None => Err(MoveError::Missing(m.id)),
                }
            }
            None => Err(MoveError::Missing(m.id)),
        }
    }

    fn execute_item<ID: PartialEq, R: Rng, PLR: Players<ID>>(
        &self,
        battle: &BattleData,
        random: &mut R,
        item: &Item,
        user: &ID,
        target: PokemonIdentifier<ID>,
        players: &mut PLR,
    ) -> Result<Vec<ItemResult>, Self::ItemError> {
        match self.items.get(&item.id) {
            Some(execution) => match execution {
                BattleItemExecution::Normal(..) => {
                    // to - do: fix this function
                    log::debug!("fix OwnedPokemon::try_use_item");
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
                        .execute_item(battle, random, item, user, target, players)
                        .map_err(ItemError::Script);
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

    fn update(&self) {}
}
