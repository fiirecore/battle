use core::{
    cell::{Cell, Ref, RefCell, RefMut},
    hash::Hash,
    iter::FromIterator,
};

use pokedex::moves::{Move, MoveTarget};
use rand::{Rng, prelude::IteratorRandom};

use crate::{engine::{Players, BattlePokemon}, pokemon::PokemonIdentifier};

use super::player::BattlePlayer;

pub struct BattleMap<K: Eq + Hash, V>(hashbrown::HashMap<K, (Properties, RefCell<V>)>);

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

impl<'d, ID: Eq + Hash + Clone, R: Rng, const AS: usize> Players<'d, ID, R>
    for BattleMap<ID, BattlePlayer<'d, ID, AS>>
{
    fn create_targets(&self, user: &PokemonIdentifier<ID>, m: &Move, targeting: Option<PokemonIdentifier<ID>>, random: &mut R) -> Vec<PokemonIdentifier<ID>> {
        match m.target {
            MoveTarget::Any => match targeting {
                Some(id) => vec![id],
                None => match self
                    .values()
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
                {
                    Some(id) => vec![id.clone()],
                    None => Vec::new(), //return Err(DefaultMoveError::NoTarget),
                },
            },
            MoveTarget::Ally => match targeting {
                Some(id) => match id.team() == user.team() && id.index() != user.index() {
                    true => vec![id],
                    false => todo!(),
                },
                None => todo!(),
            },
            MoveTarget::Allies => todo!(),
            MoveTarget::UserOrAlly => todo!(),
            MoveTarget::UserAndAllies => todo!(),
            MoveTarget::User => todo!(),
            MoveTarget::Opponent => match targeting {
                Some(id) => match id.team() != user.team() {
                    true => vec![id],
                    false => todo!(),
                },
                None => match self
                    .values()
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
                {
                    Some(id) => vec![id],
                    None => return Vec::new(), //return Err(DefaultMoveError::NoTarget),
                },
            },
            MoveTarget::AllOpponents => todo!(),
            MoveTarget::RandomOpponent => todo!(),
            MoveTarget::AllOtherPokemon => todo!(),
            MoveTarget::AllPokemon => todo!(),
            MoveTarget::None => todo!(),
        }
    }

    fn get(&self, id: &PokemonIdentifier<ID>) -> Option<&BattlePokemon<'d>> {
        if let Some(p) = self.get(id.team()) {
            if let Some(p) = p.party.active(id.index()) {
                // i think this is safe
                let p2 = unsafe { & *((&p.p) as *const BattlePokemon<'d>) };
                return Some(p2);
            }
        }
        None
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
