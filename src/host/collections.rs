use core::{
    cell::{Cell, Ref, RefCell, RefMut},
    hash::Hash,
    iter::FromIterator,
    ops::Deref,
};

use pokedex::{
    item::Item,
    moves::{Move, MoveTarget},
    pokemon::Pokemon,
};
use rand::{prelude::IteratorRandom, Rng};

use crate::{
    engine::{BattlePokemon, Players},
    party::ActivePokemon,
    pokemon::PokemonIdentifier,
};

use super::player::BattlePlayer;

pub struct BattleMap<K: Eq + Hash, V>(std::collections::HashMap<K, (Properties, RefCell<V>)>);

pub struct Properties {
    pub active: Cell<bool>,
}

impl<K: Eq + Hash, V> BattleMap<K, V> {
    pub fn get(&self, k: &K) -> Option<Ref<V>> {
        self.0
            .get(k)
            .filter(|(p, ..)| p.active.get())
            .map(|(.., v)| v.try_borrow().ok())
            .flatten()
    }

    pub fn get_mut(&self, k: &K) -> Option<RefMut<V>> {
        self.0
            .get(k)
            .filter(|(p, ..)| p.active.get())
            .map(|(.., v)| v.try_borrow_mut().ok())
            .flatten()
    }

    // pub fn len(&self) -> usize {
    //     self.0.len()
    // }

    pub fn active(&self) -> usize {
        self.0.values().filter(|(p, ..)| p.active.get()).count()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, Ref<V>)> {
        self.0
            .iter()
            .filter(|(.., (p, ..))| p.active.get())
            .flat_map(|(k, (.., v))| v.try_borrow().map(|v| (k, v)))
    }

    pub fn iter_mut(&self) -> impl Iterator<Item = (&K, RefMut<V>)> {
        self.0
            .iter()
            .filter(|(.., (p, ..))| p.active.get())
            .flat_map(|(k, (.., v))| v.try_borrow_mut().map(|v| (k, v)))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0
            .iter()
            .filter(|(.., (p, ..))| p.active.get())
            .map(|(k, ..)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = Ref<V>> {
        self.0
            .values()
            .filter(|(p, ..)| p.active.get())
            .flat_map(|(.., v)| v.try_borrow())
    }

    pub fn values_mut(&self) -> impl Iterator<Item = RefMut<V>> {
        self.0
            .values()
            .filter(|(p, ..)| p.active.get())
            .flat_map(|(.., v)| v.try_borrow_mut())
    }

    pub fn all_values_mut(&self) -> impl Iterator<Item = RefMut<V>> {
        self.0.values().flat_map(|(.., v)| v.try_borrow_mut())
    }

    pub fn deactivate(&self, k: &K) -> Option<RefMut<V>> {
        if let Some((properties, v)) = self.0.get(k) {
            properties.active.set(false);
            v.try_borrow_mut().ok()
        } else {
            None
        }
    }

    // pub fn inactives(&self)

    // pub fn inactives_mut(&self)

    pub fn clear(&mut self) {}
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            active: Cell::new(true),
        }
    }
}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for BattleMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(FromIterator::from_iter(
            iter.into_iter()
                .map(|(k, v)| (k, (Default::default(), RefCell::new(v)))),
        ))
    }
}

impl<
        ID: Eq + Hash + Clone,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
    > BattleMap<ID, BattlePlayer<ID, P, M, I>>
{
    fn ally(
        &self,
        random: &mut impl Rng,
        user: &PokemonIdentifier<ID>,
        id: PokemonIdentifier<ID>,
    ) -> Vec<PokemonIdentifier<ID>> {
        self.get(id.team())
            .map(|p| {
                p.party
                    .active
                    .iter()
                    .map(|a| a.as_ref().map(ActivePokemon::index))
                    .flatten()
                    .filter(|i| *i != user.index())
                    .choose(random)
                    .map(|i| PokemonIdentifier(id.0, i))
            })
            .flatten()
            .map(|i| vec![i])
            .unwrap_or_default()
    }

    fn opponent(
        &self,
        random: &mut impl Rng,
        user: &PokemonIdentifier<ID>,
    ) -> Vec<PokemonIdentifier<ID>> {
        self.values()
            .filter(|p| p.id() != user.team() && !p.party.all_fainted())
            .choose(random)
            .map(|p| {
                p.party
                    .active
                    .iter()
                    .enumerate()
                    .filter(|(.., p)| p.is_some())
                    .map(|(i, ..)| i)
                    .choose(random)
                    .map(|i| PokemonIdentifier(p.id().clone(), i))
            })
            .flatten()
            .map(|i| vec![i])
            .unwrap_or_default()
    }

    fn allies(&self, user: &PokemonIdentifier<ID>) -> Vec<PokemonIdentifier<ID>> {
        self.get(user.team())
            .map(|p| {
                p.party
                    .active
                    .iter()
                    .flatten()
                    .map(ActivePokemon::index)
                    .filter(|i| *i != user.index())
                    .map(|u| PokemonIdentifier(user.0.clone(), u))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn all_opponents(&self, user: &PokemonIdentifier<ID>) -> Vec<PokemonIdentifier<ID>> {
        self.values()
            .filter(|p| p.id() != user.team())
            .flat_map(|p| {
                p.party
                    .active
                    .iter()
                    .flatten()
                    .map(ActivePokemon::index)
                    .map(|i| PokemonIdentifier(p.id().clone(), i))
                    // VVV bad code VVV
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect()
    }

    fn all_other_pokemon(&self, user: &PokemonIdentifier<ID>) -> Vec<PokemonIdentifier<ID>> {
        let mut v = self.allies(user);
        v.extend_from_slice(&self.all_opponents(user));
        v
    }

    fn user_and_allies(&self, user: &PokemonIdentifier<ID>) -> Vec<PokemonIdentifier<ID>> {
        self.get(user.team())
            .into_iter()
            .map(move |p| {
                p.party
                    .active
                    .iter()
                    .flatten()
                    .map(ActivePokemon::index)
                    .map(|i| PokemonIdentifier(p.id().clone(), i))
                    // VVV bad code VVV
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .flatten()
            .collect()
    }

    fn random_user_or_allies(
        &self,
        random: &mut impl Rng,
        user: &PokemonIdentifier<ID>,
    ) -> Vec<PokemonIdentifier<ID>> {
        self.user_or_allies(random.gen_bool(0.5), user)
    }

    fn user_or_allies(
        &self,
        is_user: bool,
        user: &PokemonIdentifier<ID>,
    ) -> Vec<PokemonIdentifier<ID>> {
        match is_user {
            true => vec![user.clone()],
            false => self
                .get(user.team())
                .into_iter()
                .flat_map(|p| {
                    p.party
                        .active
                        .iter()
                        .flatten()
                        .map(ActivePokemon::index)
                        .filter(|i| *i != user.index())
                        .map(|i| PokemonIdentifier(user.0.clone(), i))
                        // VVV bad code VVV
                        .collect::<Vec<_>>()
                        .into_iter()
                })
                .collect(),
        }
    }
}

impl<
        ID: Eq + Hash + Clone,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
    > Players<ID, P, M, I> for BattleMap<ID, BattlePlayer<ID, P, M, I>>
{
    fn create_targets<R: Rng>(
        &self,
        user: &PokemonIdentifier<ID>,
        m: &Move,
        targeting: Option<PokemonIdentifier<ID>>,
        random: &mut R,
    ) -> Vec<PokemonIdentifier<ID>> {
        match m.target {
            MoveTarget::Any => match targeting {
                Some(id) => vec![id],
                None => match self
                    .values()
                    .filter(|p| {
                        !(p.id() == user.team() && m.power.is_some()) && !p.party.all_fainted()
                    })
                    .choose(random)
                    .map(|p| {
                        p.party
                            .active
                            .iter()
                            .enumerate()
                            .filter(|(.., p)| p.is_some())
                            .map(|(i, ..)| i)
                            .choose(random)
                            .map(|i| PokemonIdentifier(p.id().clone(), i))
                    })
                    .flatten()
                {
                    Some(id) => vec![id],
                    None => Vec::new(), //return Err(DefaultMoveError::NoTarget),
                },
            },
            MoveTarget::Ally => match targeting {
                Some(id) => match id.team() == user.team() && id.index() != user.index() {
                    true => vec![id],
                    false => self.ally(random, user, id),
                },
                None => self.ally(random, user, user.clone()),
            },
            MoveTarget::Allies => self.allies(user),
            MoveTarget::UserOrAlly => match targeting {
                Some(id) => match id.team() == user.team() {
                    true => self.user_or_allies(id.index() == user.index(), user),
                    false => self.random_user_or_allies(random, user),
                },
                None => self.random_user_or_allies(random, user),
            },
            MoveTarget::UserAndAllies => self.user_and_allies(user),
            MoveTarget::User => vec![user.clone()],
            MoveTarget::Opponent => match targeting {
                Some(id) => match id.team() != user.team() {
                    true => vec![id],
                    false => self.opponent(random, user),
                },
                None => self.opponent(random, user),
            },
            MoveTarget::AllOpponents => self.all_opponents(user),
            MoveTarget::RandomOpponent => self.opponent(random, user),
            MoveTarget::AllOtherPokemon => self.all_other_pokemon(user),
            MoveTarget::AllPokemon => self
                .values()
                .flat_map(|p| {
                    p.party
                        .active
                        .iter()
                        .flatten()
                        .map(ActivePokemon::index)
                        .map(|i| PokemonIdentifier(p.id().clone(), i))
                        // VVV bad code VVV
                        .collect::<Vec<_>>()
                        .into_iter()
                })
                .collect(),
            MoveTarget::None => vec![],
        }
    }

    fn get(&self, id: &PokemonIdentifier<ID>) -> Option<&BattlePokemon<P, M, I>> {
        if let Some(p) = BattleMap::get(&self, id.team()) {
            if let Some(p) = p.party.active(id.index()) {
                // i think this is safe
                let p2 = unsafe { &*((&p.p) as *const BattlePokemon<P, M, I>) };
                return Some(p2);
            }
        }
        None
    }

    fn get_mut(&mut self, id: &PokemonIdentifier<ID>) -> Option<&mut BattlePokemon<P, M, I>> {
        if let Some(mut p) = BattleMap::get_mut(&self, id.team()) {
            if let Some(p) = p.party.active_mut(id.index()) {
                // i think this is safe
                let p2 = unsafe { &mut *((&mut p.p) as *mut BattlePokemon<P, M, I>) };
                return Some(p2);
            }
        }
        None
    }

    fn take(&mut self, id: &PokemonIdentifier<ID>) -> Option<BattlePokemon<P, M, I>> {
        match BattleMap::get_mut(&self, id.team()) {
            Some(mut player) => player.party.take(id.index()).map(|p| p.p),
            None => None,
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
