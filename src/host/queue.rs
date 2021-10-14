use core::{cmp::Reverse, hash::Hash};
use rand::Rng;
use std::collections::BTreeMap;

use pokedex::{
    moves::Priority,
    pokemon::stat::{BaseStat, StatType},
};

use crate::{
    moves::BattleMove, party::BattleParty, player::BattlePlayer, pokemon::PokemonIndex,
    BattleEndpoint, Indexed,
};

use super::collection::BattleMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovePriority<ID: Ord> {
    First(ID, usize),
    Second(Reverse<Priority>, Reverse<BaseStat>, Option<u16>),
}

pub fn move_queue<ID: Clone + Ord + Hash, E: BattleEndpoint<ID, AS>, R: Rng, const AS: usize>(
    players: &mut BattleMap<ID, BattlePlayer<ID, E, AS>>,
    random: &mut R,
) -> Vec<Indexed<ID, BattleMove<ID>>> {
    let mut queue = BTreeMap::new();

    for mut player in players.values_mut() {
        queue_player(&mut queue, &mut player.party, random)
    }

    queue.into_values().collect()
}

fn queue_player<ID: Clone + Ord, R: Rng, const AS: usize>(
    queue: &mut BTreeMap<MovePriority<ID>, Indexed<ID, BattleMove<ID>>>,
    party: &mut BattleParty<ID, AS>,
    random: &mut R,
) {
    for index in 0..AS {
        if let Some(pokemon) = party.active.get_mut(index).map(Option::as_mut).flatten() {
            if let Some(action) = pokemon.queued_move.take() {
                if let Some(instance) = party.active(index) {
                    let pokemon = PokemonIndex(party.id().clone(), index);

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
