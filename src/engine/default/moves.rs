use core::ops::Deref;

use rand::Rng;

use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::{Ailment, AilmentLength},
    item::Item,
    moves::Move,
    pokemon::Pokemon,
};

use crate::{
    moves::{damage::DamageKind, Percent},
    pokemon::{
        stat::{BattleStatType, Stage},
        Indexed,
    },
};

use super::{BattlePokemon, MoveResult};

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
    pub fn size(&self) -> usize {
        match self {
            Self::Actions(actions) => actions.iter().map(MoveUse::size).sum(),
            Self::Script | Self::None => 1,
        }
    }
}

impl MoveUse {
    pub fn size(&self) -> usize {
        match self {
            Self::Chance(uses, ..) => uses.iter().map(Self::size).sum(),
            Self::Drain(..) => 2,
            _ => 1,
        }
    }
}

pub fn move_usage<
    'd,
    ID: Clone,
    R: Rng,
    P: Deref<Target = Pokemon>,
    M: Deref<Target = Move>,
    I: Deref<Target = Item>,
>(
    user: &Indexed<ID, &BattlePokemon<P, M, I>>,
    random: &mut R,
    results: &mut Vec<Indexed<ID, MoveResult>>,
    actions: &[MoveUse],
    m: &Move,
    Indexed(target_id, target): Indexed<ID, &BattlePokemon<P, M, I>>,
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
                if target.ailment.is_none() && random.gen_bool(*chance as f64 / 100.0) {
                    results.push(Indexed(
                        target_id.clone(),
                        MoveResult::Ailment(length.init(*status, random)),
                    ));
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
