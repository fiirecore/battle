use core::hash::Hash;
use hashbrown::HashMap;
use rand::{prelude::IteratorRandom, Rng};
use std::error::Error;

use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::{Ailment, AilmentLength},
    moves::{Move, MoveId, MoveTarget},
};

use crate::{
    host::pokemon::BattlePokemon,
    moves::{damage::DamageKind, Percent},
    pokemon::{
        stat::{BattleStatType, Stage},
        PokemonIdentifier,
    },
    BattleEndpoint, Indexed,
};

use crate::host::{collections::BattleMap, player::BattlePlayer};

use super::{MoveEngine, MoveResult};

#[cfg(feature = "default_scripting")]
pub mod scripting;

pub type Moves = HashMap<MoveId, MoveExecution>;

pub struct DefaultMoveEngine {
    pub moves: Moves,
    #[cfg(feature = "default_scripting")]
    pub scripting: scripting::DefaultScriptingEngine,
}

impl DefaultMoveEngine {
    pub fn new<'d, ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        Self {
            moves: Default::default(),
            #[cfg(feature = "default_scripting")]
            scripting: scripting::DefaultScriptingEngine::new::<ID, R>(),
        }
    }
}

impl MoveEngine for DefaultMoveEngine {
    type Error = DefaultMoveError;

    fn execute<
        'd,
        ID: Clone + Hash + Eq + 'static,
        R: Rng + Clone + 'static,
        E: BattleEndpoint<ID, AS>,
        const AS: usize,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon<'d>>,
        targeting: Option<PokemonIdentifier<ID>>,
        players: &BattleMap<ID, BattlePlayer<'d, ID, E, AS>>,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error> {
        match self.moves.get(&m.id) {
            Some(usage) => {
                let targets = match m.target {
                    MoveTarget::Any => match targeting {
                        Some(id) => vec![id],
                        None => match players
                            .keys()
                            .map(|id| (id, players.get(id).unwrap()))
                            .filter(|(.., p)| !p.party.all_fainted())
                            .choose(random)
                            .map(|(id, p)| {
                                p.party
                                    .active
                                    .iter()
                                    .enumerate()
                                    .filter(|(.., p)| p.is_some())
                                    .map(|(i, ..)| i)
                                    .choose(random)
                                    .map(|i| PokemonIdentifier(id.clone(), i))
                            })
                            .flatten()
                        {
                            Some(id) => vec![id],
                            None => return Err(DefaultMoveError::NoTarget),
                        },
                    },
                    MoveTarget::Ally => match targeting {
                        Some(id) => {
                            match id.team() == user.0.team() && id.index() != user.0.index() {
                                true => vec![id],
                                false => todo!(),
                            }
                        }
                        None => todo!(),
                    },
                    MoveTarget::Allies => todo!(),
                    MoveTarget::UserOrAlly => todo!(),
                    MoveTarget::UserAndAllies => todo!(),
                    MoveTarget::User => todo!(),
                    MoveTarget::Opponent => match targeting {
                        Some(id) => match id.team() != user.0.team() {
                            true => vec![id],
                            false => todo!(),
                        },
                        None => match players
                            .keys()
                            .filter(|id| *id != user.0.team())
                            .map(|id| (id, players.get(id).unwrap()))
                            .filter(|(.., p)| !p.party.all_fainted())
                            .choose(random)
                            .map(|(id, p)| {
                                p.party
                                    .active
                                    .iter()
                                    .enumerate()
                                    .filter(|(.., p)| p.is_some())
                                    .map(|(i, ..)| i)
                                    .choose(random)
                                    .map(|i| PokemonIdentifier(id.clone(), i))
                            })
                            .flatten()
                        {
                            Some(id) => vec![id],
                            None => return Err(DefaultMoveError::NoTarget),
                        },
                    },
                    MoveTarget::AllOpponents => todo!(),
                    MoveTarget::RandomOpponent => todo!(),
                    MoveTarget::AllOtherPokemon => todo!(),
                    MoveTarget::AllPokemon => todo!(),
                    MoveTarget::None => todo!(),
                };

                match &usage {
                    MoveExecution::Actions(actions) => {
                        let mut results = Vec::new();
                        for target_id in targets {
                            match players.get(target_id.team()) {
                                Some(target) => match target.party.active(target_id.index()) {
                                    Some(target) => match user.1.throw_move(random, m) {
                                        true => {
                                            results.reserve(usage.len());
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
                                    None => log::warn!(
                                        "Cannot get active pokemon #{} from user {}",
                                        target_id.index(),
                                        target.name()
                                    ),
                                },
                                None => todo!(),
                            }
                        }
                        return Ok(results);
                    }
                    MoveExecution::Script => {
                        #[cfg(feature = "default_scripting")]
                        return self.scripting.execute(random, m, user, targets, players);
                        #[cfg(not(feature = "default_scripting"))]
                        return Err(DefaultMoveError::NoScriptEngine);
                    }
                    MoveExecution::None => return Err(DefaultMoveError::Missing),
                }
            }
            None => Err(DefaultMoveError::Missing),
        }
    }
}

pub fn move_usage<'d, ID: Clone, R: Rng>(
    user: &Indexed<ID, &BattlePokemon<'d>>,
    random: &mut R,
    results: &mut Vec<Indexed<ID, MoveResult>>,
    actions: &[MoveUse],
    m: &Move,
    Indexed(target_id, target): Indexed<ID, &BattlePokemon<'d>>,
) {
    for action in actions {
        match action {
            MoveUse::Damage(kind) => {
                results.push(Indexed(
                    target_id.clone(),
                    MoveResult::Damage(target.damage_kind(
                        random,
                        target,
                        *kind,
                        m.category,
                        m.pokemon_type,
                        m.crit_rate,
                    )),
                ));
            }
            MoveUse::Ailment(status, length, chance) => {
                if target.ailment.is_none() {
                    if random.gen_bool(*chance as f64 / 100.0) {
                        results.push(Indexed(
                            target_id.clone(),
                            MoveResult::Ailment(length.init(*status, random)),
                        ));
                    }
                }
            }
            MoveUse::Drain(kind, percent) => {
                let result = user.1.damage_kind(
                    random,
                    target,
                    *kind,
                    m.category,
                    m.pokemon_type,
                    m.crit_rate,
                );

                let healing = (result.damage as f32 * *percent as f32 / 100.0) as i16;

                results.push(Indexed(target_id.clone(), MoveResult::Damage(result)));
                results.push(Indexed(user.0.clone(), MoveResult::Heal(healing)))
            }
            MoveUse::Stat(stat, stage) => {
                if target.stages.can_change(*stat, *stage) {
                    results.push(Indexed(target_id.clone(), MoveResult::Stat(*stat, *stage)));
                }
            }
            // MoveUseType::Linger(..) => {
            // 	results.insert(target.instance, Some(MoveAction::Todo));
            // }
            MoveUse::Flinch => results.push(Indexed(target_id.clone(), MoveResult::Flinch)),
            MoveUse::Chance(actions, chance) => {
                if random.gen_range(0..=100) < *chance {
                    move_usage(
                        user,
                        random,
                        results,
                        actions,
                        m,
                        Indexed(target_id.clone(), target),
                    );
                }
            }
        }
    }
}

// let targets: Vec<TargetLocation<ID>> = match target {
//     MoveTargetInstance::Any(id, index) => match user.id() == id {
//         true => match index == &instance.0.index() {
//             true => TargetLocation::user().collect(),
//             false => TargetLocation::team(*index).collect(),
//         },
//         false => TargetLocation::opponent(id.clone(), *index).collect(),
//     },
//     MoveTargetInstance::Ally(index) => {
//         TargetLocation::team(*index).collect()
//     }
//     MoveTargetInstance::Allies => {
//         TargetLocation::allies::<AS>(instance.0.index()).collect()
//     }
//     MoveTargetInstance::UserOrAlly(index) => {
//         match index == &instance.0.index() {
//             true => TargetLocation::user().collect(),
//             false => TargetLocation::team(*index).collect(),
//         }
//     }
//     MoveTargetInstance::User => TargetLocation::user().collect(),
//     MoveTargetInstance::Opponent(id, index) => {
//         TargetLocation::opponent(id.clone(), *index).collect()
//     }
//     MoveTargetInstance::AllOpponents(id) => {
//         TargetLocation::opponents::<AS>(id).collect()
//     }
//     MoveTargetInstance::RandomOpponent(id) => {
//         TargetLocation::opponent(id.clone(), random.gen_range(0..AS))
//             .collect()
//     }
//     MoveTargetInstance::AllOtherPokemon(id) => {
//         TargetLocation::all_other_pokemon::<AS>(
//             id,
//             instance.0.index(),
//         )
//         .collect()
//     }
//     MoveTargetInstance::None => {
//         if let Some(user) = user.party.active(instance.0.index())
//         {
//             warn!(
//                 "Could not use move '{}' because it has no target implemented.",
//                 user
//                     .moves
//                     .get(move_index)
//                     .map(|i| i.0.name())
//                     .unwrap_or("Unknown")
//             );
//         }
//         vec![]
//     }
//     MoveTargetInstance::UserAndAllies => {
//         TargetLocation::user_and_allies::<AS>(instance.0.index())
//             .collect()
//     }
//     MoveTargetInstance::AllPokemon(id) => {
//         TargetLocation::all_pokemon::<AS>(id, instance.0.index())
//             .collect()
//     }
// };

#[derive(Debug)]
pub enum DefaultMoveError {
    #[cfg(feature = "default_scripting")]
    Script(Box<rhai::EvalAltResult>),
    #[cfg(not(feature = "default_scripting"))]
    NoScriptEngine,
    Missing,
    NoTarget,
}

impl Error for DefaultMoveError {}

impl core::fmt::Display for DefaultMoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "default_scripting")]
            Self::Script(err) => core::fmt::Display::fmt(err, f),
            other => core::fmt::Debug::fmt(other, f),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MoveExecution {
    /// Load a vector of actions
    Actions(Vec<MoveUse>),
    /// Use a script defined in the instance of the object that uses this
    Script,
    /// Placeholder to show that object does not have a defined use yet.
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum MoveUse {
    Damage(DamageKind),
    Ailment(Ailment, AilmentLength, Percent),
    Drain(DamageKind, i8),
    Stat(BattleStatType, Stage),
    Flinch,
    Chance(Vec<Self>, Percent),
}

impl MoveExecution {
    pub fn len(&self) -> usize {
        match self {
            Self::Actions(actions) => actions.iter().map(MoveUse::len).sum(),
            Self::Script | Self::None => 1,
        }
    }
}

impl MoveUse {
    pub fn len(&self) -> usize {
        match self {
            Self::Chance(uses, ..) => uses.iter().map(Self::len).sum(),
            Self::Drain(..) => 2,
            _ => 1,
        }
    }
}
