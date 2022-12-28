use core::{cmp::Reverse, hash::Hash};
use rand::Rng;
use std::collections::BTreeMap;

use pokedex::pokemon::stat::{BaseStat, StatType};

use crate::{
    pokemon::{Indexed, TeamIndex},
    select::BattleSelection, moves::Priority, engine::{BattlePokemon, BattleEngine},
};

use super::pokemon::{ActiveBattlePokemon};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovePriority<ID: Ord> {
    First(ID, usize),
    Second(Reverse<Priority>, Reverse<BaseStat>, Option<u16>),
}

pub fn queue_player<ID: Clone + Ord + Hash + Send + Sync + 'static, T: Send + Sync + 'static, R: Rng>(
    engine: &impl BattleEngine<ID, T>,
    queue: &mut BTreeMap<MovePriority<ID>, Indexed<ID, BattleSelection<ID>>>,
    id: &ID,
    active: &mut [Option<ActiveBattlePokemon<ID>>],
    party: &mut [BattlePokemon],
    random: &mut R,
) {
    for index in 0..active.len() {
        if let Some(active) = active.get_mut(index).and_then(Option::as_mut) {
            if let Some(action) = active.queued_move.take() {
                if let Some(instance) = party.get(active.index) {
                    let pokemon = TeamIndex(id.clone(), index);

                    let mut priority = match action {
                        BattleSelection::Move(id, ..) => MovePriority::Second(
                            Reverse(
                                instance
                                    .moves
                                    .iter()
                                    .find(|m| m.id() == &id)
                                    .and_then(|m| engine.get_move(m.id()).map(|m| m.priority))
                                    .unwrap_or_default(),
                            ),
                            Reverse(instance.stat(StatType::Speed)),
                            None,
                        ),
                        _ => MovePriority::First(id.clone(), index),
                    };

                    fn tie_break<ID: Ord, R: Rng>(
                        queue: &mut BTreeMap<MovePriority<ID>, Indexed<ID, BattleSelection<ID>>>,
                        random: &mut R,
                        priority: &mut MovePriority<ID>,
                    ) {
                        if let MovePriority::Second(.., shift) = priority {
                            *shift = Some(random.gen());
                        }
                        if queue.contains_key(priority) {
                            tie_break(queue, random, priority);
                        }
                    }

                    if queue.contains_key(&priority) {
                        tie_break(queue, random, &mut priority);
                    }

                    queue.insert(priority, Indexed(pokemon, action));
                }
            }
        }
    }
}
