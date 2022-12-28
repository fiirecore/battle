use std::marker::PhantomData;

use rand::{seq::IteratorRandom, Rng};

use battle::{
    engine::PlayerQuery,
    pokemon::TeamIndex,
    party::ActivePokemon,
    moves::{BattleMove, MoveTarget, MoveCategory},
};

struct QueryTargeter<ID: PartialEq, T> {
    _p: PhantomData<(ID, T)>,
}

impl<ID: PartialEq + Clone, T> QueryTargeter<ID, T> {
    fn ally(
        query: &PlayerQuery<ID, T>,
        random: &mut impl Rng,
        user: &TeamIndex<ID>,
        id: TeamIndex<ID>,
    ) -> Vec<TeamIndex<ID>> {
        query
            .iter()
            .find(|p| &p.party.id == id.team())
            .and_then(|p| {
                p.party
                    .active
                    .iter()
                    .filter_map(|a| a.as_ref().map(ActivePokemon::index))
                    .filter(|i| *i != user.index())
                    .choose(random)
                    .map(|i| TeamIndex(id.0, i))
            })
            .map(|i| vec![i])
            .unwrap_or_default()
    }

    fn opponent(
        query: &PlayerQuery<ID, T>,
        random: &mut impl Rng,
        user: &TeamIndex<ID>,
    ) -> Vec<TeamIndex<ID>> {
        query
            .iter()
            .filter(|p| p.id() != user.team() && !p.party.all_fainted())
            .choose(random)
            .and_then(|p| {
                p.party
                    .active
                    .iter()
                    .enumerate()
                    .filter(|(.., p)| p.is_some())
                    .map(|(i, ..)| i)
                    .choose(random)
                    .map(|i| TeamIndex(p.id().clone(), i))
            })
            .map(|i| vec![i])
            .unwrap_or_default()
    }

    fn allies(query: &PlayerQuery<ID, T>, user: &TeamIndex<ID>) -> Vec<TeamIndex<ID>> {
        query
            .iter()
            .find(|p| &p.party.id == user.team())
            .map(|p| {
                p.party
                    .active
                    .iter()
                    .flatten()
                    .map(ActivePokemon::index)
                    .filter(|i| *i != user.index())
                    .map(|u| TeamIndex(user.0.clone(), u))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn all_opponents(query: &PlayerQuery<ID, T>, user: &TeamIndex<ID>) -> Vec<TeamIndex<ID>> {
        query
            .iter()
            .filter(|p| p.id() != user.team())
            .flat_map(|p| {
                p.party
                    .active
                    .iter()
                    .flatten()
                    .map(ActivePokemon::index)
                    .map(|i| TeamIndex(p.id().clone(), i))
                    // VVV bad code VVV
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect()
    }

    fn all_other_pokemon(query: &PlayerQuery<ID, T>, user: &TeamIndex<ID>) -> Vec<TeamIndex<ID>> {
        let mut v = Self::allies(query, user);
        v.extend(Self::all_opponents(query, user));
        v
    }

    fn user_and_allies(query: &PlayerQuery<ID, T>, user: &TeamIndex<ID>) -> Vec<TeamIndex<ID>> {
        query
            .iter()
            .find(|p| &p.party.id == user.team())
            .into_iter()
            .flat_map(move |p| {
                p.party
                    .active
                    .iter()
                    .flatten()
                    .map(ActivePokemon::index)
                    .map(|i| TeamIndex(p.id().clone(), i))
                    // VVV bad code VVV
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect()
    }

    fn random_user_or_allies(
        query: &PlayerQuery<ID, T>,
        random: &mut impl Rng,
        user: &TeamIndex<ID>,
    ) -> Vec<TeamIndex<ID>> {
        Self::user_or_allies(query, random.gen_bool(0.5), user)
    }

    fn user_or_allies(
        query: &PlayerQuery<ID, T>,
        is_user: bool,
        user: &TeamIndex<ID>,
    ) -> Vec<TeamIndex<ID>> {
        match is_user {
            true => vec![user.clone()],
            false => query
                .iter()
                .find(|p| &p.party.id == user.team())
                .into_iter()
                .flat_map(|p| {
                    p.party
                        .active
                        .iter()
                        .flatten()
                        .map(ActivePokemon::index)
                        .filter(|i| *i != user.index())
                        .map(|i| TeamIndex(user.0.clone(), i))
                        // VVV bad code VVV
                        .collect::<Vec<_>>()
                        .into_iter()
                })
                .collect(),
        }
    }
}

pub fn create_targets<ID: PartialEq + Clone, T, R: Rng>(
    query: &PlayerQuery<ID, T>,
    user: &TeamIndex<ID>,
    m: &BattleMove,
    targeting: Option<&TeamIndex<ID>>,
    random: &mut R,
) -> Vec<TeamIndex<ID>> {
    match m.target {
        MoveTarget::Any => match targeting {
            Some(id) => vec![id.clone()],
            None => match query
                .iter()
                .filter(|p| {
                    !(p.id() == user.team() && m.category != MoveCategory::Status)
                        && !p.party.all_fainted()
                })
                .choose(random)
                .and_then(|p| {
                    p.party
                        .active
                        .iter()
                        .enumerate()
                        .filter(|(.., p)| p.is_some())
                        .map(|(i, ..)| i)
                        .choose(random)
                        .map(|i| TeamIndex(p.id().clone(), i))
                }) {
                Some(id) => vec![id],
                None => Vec::new(), //return Err(DefaultMoveError::NoTarget),
            },
        },
        MoveTarget::Ally => match targeting {
            Some(id) => match id.team() == user.team() && id.index() != user.index() {
                true => vec![id.clone()],
                false => QueryTargeter::ally(query, random, user, id.clone()),
            },
            None => QueryTargeter::ally(query, random, user, user.clone()),
        },
        MoveTarget::Allies => QueryTargeter::allies(query, user),
        MoveTarget::UserOrAlly => match targeting {
            Some(id) => match id.team() == user.team() {
                true => QueryTargeter::user_or_allies(query, id.index() == user.index(), user),
                false => QueryTargeter::random_user_or_allies(query, random, user),
            },
            None => QueryTargeter::random_user_or_allies(query, random, user),
        },
        MoveTarget::UserAndAllies => QueryTargeter::user_and_allies(query, user),
        MoveTarget::User => vec![user.clone()],
        MoveTarget::Opponent => match targeting {
            Some(id) => match id.team() != user.team() {
                true => vec![id.clone()],
                false => QueryTargeter::opponent(query, random, user),
            },
            None => QueryTargeter::opponent(query, random, user),
        },
        MoveTarget::AllOpponents => QueryTargeter::all_opponents(query, user),
        MoveTarget::RandomOpponent => QueryTargeter::opponent(query, random, user),
        MoveTarget::AllOtherPokemon => QueryTargeter::all_other_pokemon(query, user),
        MoveTarget::AllPokemon => query
            .iter()
            .flat_map(|p| {
                p.party
                    .active
                    .iter()
                    .flatten()
                    .map(ActivePokemon::index)
                    .map(|i| TeamIndex(p.id().clone(), i))
                    // VVV bad code VVV
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect(),
        MoveTarget::None => vec![],
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
//         TargetLocation::allies::<>(instance.0.index()).collect()
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
//         TargetLocation::opponents::<>(id).collect()
//     }
//     MoveTargetInstance::RandomOpponent(id) => {
//         TargetLocation::opponent(id.clone(), random.gen_range(0..AS))
//             .collect()
//     }
//     MoveTargetInstance::AllOtherPokemon(id) => {
//         TargetLocation::all_other_pokemon::<>(
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
//         TargetLocation::user_and_allies::<>(instance.0.index())
//             .collect()
//     }
//     MoveTargetInstance::AllPokemon(id) => {
//         TargetLocation::all_pokemon::<>(id, instance.0.index())
//             .collect()
//     }
// };
