use std::{cmp::Reverse, collections::BTreeMap};

use pokedex::{
    moves::Priority,
    pokemon::stat::{BaseStat, FullStatType, StatType},
};

use crate::{
    moves::{BattleMove, BoundBattleMove},
    party::BattleParty,
    pokemon::PokemonIndex,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovePriority {
    First,
    Second(Reverse<Priority>, Reverse<BaseStat>), // priority, speed <- fix last, make who goes first random
}

pub fn move_queue<ID: Copy>(
    player1: &mut BattleParty<ID>,
    player2: &mut BattleParty<ID>,
) -> Vec<BoundBattleMove<ID>> {
    let mut queue = BTreeMap::new();

    queue_player(&mut queue, player1);
    queue_player(&mut queue, player2);

    queue.into_iter().map(|(_, i)| i).collect() // into_values
}

fn queue_player<ID: Copy>(
    queue: &mut BTreeMap<MovePriority, BoundBattleMove<ID>>,
    party: &mut BattleParty<ID>,
) {
    for index in 0..party.active.len() {
        if let Some(pokemon) = party.active.get_mut(index).map(Option::as_mut).flatten() {
            if let Some(action) = pokemon.queued_move.take() {
                if let Some(instance) = party.active(index) {
                    let pokemon = PokemonIndex {
                        team: party.id,
                        index,
                    };
                    queue.insert(
                        match action {
                            BattleMove::Move(index, ..) => MovePriority::Second(
                                Reverse(instance.moves[index].move_ref.priority),
                                Reverse(
                                    instance
                                        .base
                                        .get(FullStatType::Basic(StatType::Speed)),
                                ),
                            ),
                            _ => MovePriority::First,
                        },
                        BoundBattleMove { pokemon, action },
                    );
                }
            }
        }
    }
}