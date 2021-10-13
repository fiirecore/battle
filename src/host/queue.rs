use core::{cmp::Reverse, hash::Hash};
use rand::Rng;
use std::collections::BTreeMap;

use pokedex::{
    moves::Priority,
    pokemon::stat::{BaseStat, StatType},
};

use crate::{
    moves::BattleMove, party::BattleParty, player::BattlePlayer, pokemon::PokemonIndex,
    BattleEndpoint, BoundAction,
};

use super::collection::BattleMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovePriority<ID: Ord> {
    First(ID, usize),
    Second(Reverse<Priority>, Reverse<BaseStat>, Option<u16>),
}

// struct Player<'d, 'a, ID: Copy + Ord, MDEX: Dex<Move>, R: Rng>(&'a mut BattleParty<'d, ID, MDEX>, &'a mut R);

pub fn move_queue<ID: Copy + Ord + Hash, E: BattleEndpoint<ID>, R: Rng>(
    players: &mut BattleMap<ID, BattlePlayer<ID, E>>,
    random: &mut R,
) -> Vec<BoundAction<ID, BattleMove>> {
    let mut queue = BTreeMap::new();

    for mut player in players.values_mut() {
        queue_player(&mut queue, &mut player.party, random)
    }

    queue.into_values().collect()
}

fn queue_player<ID: Copy + Ord, R: Rng>(
    queue: &mut BTreeMap<MovePriority<ID>, BoundAction<ID, BattleMove>>,
    party: &mut BattleParty<ID>,
    random: &mut R,
) {
    for index in 0..party.active.len() {
        if let Some(pokemon) = party.active.get_mut(index).map(Option::as_mut).flatten() {
            if let Some(action) = pokemon.queued_move.take() {
                if let Some(instance) = party.active(index) {
                    let pokemon = PokemonIndex {
                        team: party.id,
                        index,
                    };

                    let id = party.id;

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
                        _ => MovePriority::First(id, index),
                    };

                    fn tie_break<ID: Ord, R: Rng>(
                        queue: &mut BTreeMap<MovePriority<ID>, BoundAction<ID, BattleMove>>,
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

                    queue.insert(priority, BoundAction { pokemon, action });
                }
            }
        }
    }
}
