use core::hash::Hash;
use hashbrown::HashMap;
use rand::Rng;
use std::error::Error;

use serde::{Deserialize, Serialize};

use pokedex::{
    moves::{Move, MoveCategory, MoveId},
    types::PokemonType,
};

use rhai::{plugin::*, Array, Dynamic, Engine, Scope, AST, INT};

use crate::{
    moves::{
        damage::{DamageKind, DamageResult},
        engine::MoveEngine,
        Percent,
    },
    pokemon::{
        battle::{
            stat::{BattleStatType, Stage},
            BattlePokemon,
        },
        PokemonIndex,
    },
    prelude::BattleMap,
};

mod damage;
mod moves;
mod pokemon;
mod random;
mod result;

use damage::*;
use moves::*;
use pokemon::*;
use random::*;
use result::*;

use super::MoveResult;

pub type Moves = HashMap<MoveId, MoveExecution>;
pub type Scripts = HashMap<MoveId, AST>;

pub struct DefaultMoveEngine {
    pub moves: Moves,
    pub scripts: Scripts,
    pub engine: Engine,
}

impl DefaultMoveEngine {
    pub fn new<'d, ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        let mut engine = Engine::new_raw();

        engine
            .register_type_with_name::<ScriptRandom<R>>("Random")
            .register_type_with_name::<DamageResult<INT>>("Damage")
            .register_set("damage", ScriptDamage::set_damage)
            .register_get("damage", ScriptDamage::get_damage)
            .register_get("effective", ScriptDamage::effective)
            .register_type_with_name::<ScriptPokemon<ID>>("Pokemon")
            .register_fn("throw_move", ScriptPokemon::<ID>::throw_move::<R>)
            .register_fn("damage", ScriptPokemon::<ID>::get_damage::<R>)
            .register_get("hp", ScriptPokemon::<ID>::hp)
            .register_type::<ScriptMove>()
            .register_get("category", ScriptMove::get_category)
            .register_get("type", ScriptMove::get_type)
            .register_get("crit_rate", ScriptMove::get_crit_rate)
            .register_type_with_name::<MoveCategory>("Category")
            .register_type_with_name::<PokemonType>("Type")
            .register_type::<MoveResult>()
            .register_type_with_name::<ScriptMoveResult<ID>>("Result")
            .register_fn("miss", ScriptMoveResult::<ID>::miss)
            .register_fn("damage", ScriptMoveResult::<ID>::damage)
            .register_fn("drain", ScriptMoveResult::<ID>::heal);

        Self {
            moves: Default::default(),
            scripts: Default::default(),
            engine,
        }
    }
}

impl MoveEngine for DefaultMoveEngine {
    type Error = DefaultMoveError;

    fn execute<'d, ID: Clone + Hash + Eq + 'static, R: Rng + Clone + 'static, const AS: usize>(
        &mut self,
        random: &mut R,
        m: &Move,
        user: &BattlePokemon<'d>,
        targets: Vec<PokemonIndex<ID>>,
        players: &BattleMap<ID, BattlePokemon<'d>>,
    ) -> Result<HashMap<PokemonIndex<ID>, Vec<MoveResult>>, Self::Error> {
        match self.moves.get(&m.id) {
            Some(usage) => {
                let mut results = HashMap::with_capacity(targets.len());

                match &usage {
                    MoveExecution::Actions(actions) => {
                        for target in targets {
                            match target.1.throw_move(random, m) {
                                true => {
                                    results.reserve(usage.len());
                                    user.move_usage(random, &mut results, actions, m, target);
                                }
                                false => results.push((TargetLocation::User, MoveResult::Miss)),
                            }
                        }
                    }
                    MoveExecution::Script => match self.scripts.get(&m.id) {
                        Some(script) => {
                            let mut scope = Scope::new();

                            scope.push("random", ScriptRandom::new(random));
                            scope.push("move", ScriptMove::new(m));
                            scope.push("user", ScriptPokemon::<ID>::new((PokemonIndex(), user)));

                            let targets = targets
                                .into_iter()
                                .map(ScriptPokemon::new)
                                .map(Dynamic::from)
                                .collect::<Array>();

                            scope.push("targets", targets);

                            results.extend(
                                self.engine
                                    .eval_ast_with_scope::<Array>(&mut scope, script)
                                    .map_err(DefaultMoveError::Script)?
                                    .into_iter()
                                    .flat_map(Dynamic::try_cast::<ScriptMoveResult<ID>>)
                                    .map(|r| (r.0, r.1))
                                    .collect::<Vec<(TargetLocation<ID>, MoveResult)>>(),
                            );
                        }
                        None => return Err(DefaultMoveError::Missing),
                    },
                    MoveExecution::None => return Err(DefaultMoveError::Missing),
                }

                Ok(results)
            }
            None => Err(DefaultMoveError::Missing),
        }
    }
}

pub fn move_usage<ID: Clone, R: Rng>(
    user: &BattlePokemon,
    random: &mut R,
    results: &mut Vec<(TargetLocation<ID>, MoveResult)>,
    actions: &Vec<MoveUse>,
    m: &Move,
    target: (TargetLocation<ID>, &BattlePokemon),
) {
    for action in actions {
        match action {
            MoveUse::Damage(kind) => {
                results.push((
                    target.0,
                    MoveResult::Damage(target.1.damage_kind(
                        random,
                        target.1,
                        *kind,
                        m.category,
                        m.pokemon_type,
                        m.crit_rate,
                    )),
                ));
            }
            MoveUse::Ailment(status, length, chance) => {
                if target.1.ailment.is_none() {
                    if random.gen_bool(*chance as f64 / 100.0) {
                        results.push((
                            target.0,
                            MoveResult::Ailment(length.init(*status, random)),
                        ));
                    }
                }
            }
            MoveUse::Drain(kind, percent) => {
                let result = self.damage_kind(
                    random,
                    target.1,
                    *kind,
                    m.category,
                    m.pokemon_type,
                    m.crit_rate,
                );

                let healing = (result.damage as f32 * *percent as f32 / 100.0) as i16;

                results.push((target.0, MoveResult::Damage(result)));
                results.push((TargetLocation::User, MoveResult::Heal(healing)))
            }
            MoveUse::Stat(stat, stage) => {
                if target.1.stages.can_change(*stat, *stage) {
                    results.push((target.0, MoveResult::Stat(*stat, *stage)));
                }
            }
            // MoveUseType::Linger(..) => {
            // 	results.insert(target.instance, Some(MoveAction::Todo));
            // }
            MoveUse::Flinch => results.push((target.0, MoveResult::Flinch)),
            MoveUse::Chance(actions, chance) => {
                if random.gen_range(0..=100) < *chance {
                    self.move_usage(random, results, actions, m, target);
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
    Script(Box<EvalAltResult>),
    Missing,
}

impl Error for DefaultMoveError {}

impl core::fmt::Display for DefaultMoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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
