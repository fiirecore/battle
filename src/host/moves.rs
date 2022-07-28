use core::{cmp::Reverse, hash::Hash};
use rand::Rng;
use std::collections::BTreeMap;

use pokedex::{
    moves::{Priority},
    pokemon::{
        stat::{BaseStat, StatType},
    },
};

use crate::{
    moves::BattleMove,
    pokemon::{Indexed, PokemonIdentifier},
};

use super::{collections::BattleMap, party::BattleParty, player::BattlePlayer};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovePriority<ID: Ord> {
    First(ID, usize),
    Second(Reverse<Priority>, Reverse<BaseStat>, Option<u16>),
}

pub fn move_queue<ID: Clone + Ord + Hash, R: Rng, T>(
    players: &mut BattleMap<ID, BattlePlayer<ID, T>>,
    random: &mut R,
) -> Vec<Indexed<ID, BattleMove<ID>>> {
    let mut queue = BTreeMap::new();

    for mut player in players.values_mut() {
        queue_player(&mut queue, &mut player.party, random)
    }

    queue.into_values().collect()
}

fn queue_player<ID: Clone + Ord, R: Rng, T>(
    queue: &mut BTreeMap<MovePriority<ID>, Indexed<ID, BattleMove<ID>>>,
    party: &mut BattleParty<ID, T>,
    random: &mut R,
) {
    for index in 0..party.active.len() {
        if let Some(pokemon) = party.active.get_mut(index).and_then(Option::as_mut) {
            if let Some(action) = pokemon.queued_move.take() {
                if let Some(instance) = party.active(index) {
                    let pokemon = PokemonIdentifier(party.id().clone(), index);

                    let mut priority = match action {
                        BattleMove::Move(index, ..) => MovePriority::Second(
                            Reverse(
                                instance
                                    .moves
                                    .get(index)
                                    .map(|i| i.0.priority)
                                    .unwrap_or_default(),
                            ),
                            Reverse(instance.stat(StatType::Speed)),
                            None,
                        ),
                        _ => MovePriority::First(party.id().clone(), index),
                    };

                    fn tie_break<ID: Ord, R: Rng>(
                        queue: &mut BTreeMap<MovePriority<ID>, Indexed<ID, BattleMove<ID>>>,
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
