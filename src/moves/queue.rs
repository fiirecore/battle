use rand::Rng;
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
pub enum MovePriority<ID: Ord> {
    First(ID),
    Second(Reverse<Priority>, Reverse<BaseStat>, bool, ID),
}

pub fn move_queue<ID: Copy + Ord, R: Rng>(
    player1: &mut BattleParty<ID>,
    player2: &mut BattleParty<ID>,
    random: &mut R,
) -> Vec<BoundBattleMove<ID>> {
    let mut queue = BTreeMap::new();

    let tiebreaker = random.gen_bool(0.5);
    queue_player(&mut queue, player1, tiebreaker);
    queue_player(&mut queue, player2, !tiebreaker);

    queue.into_iter().map(|(_, i)| i).collect() // into_values
}

fn queue_player<ID: Copy + Ord>(
    queue: &mut BTreeMap<MovePriority<ID>, BoundBattleMove<ID>>,
    party: &mut BattleParty<ID>,
    tiebreaker: bool,
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
                    queue.insert(
                        match action {
                            BattleMove::Move(index, ..) => MovePriority::Second(
                                Reverse(instance.moves[index].priority),
                                Reverse(instance.base.get(FullStatType::Basic(StatType::Speed))),
                                tiebreaker,
                                id,
                            ),
                            _ => MovePriority::First(id),
                        },
                        BoundBattleMove { pokemon, action },
                    );
                }
            }
        }
    }
}
