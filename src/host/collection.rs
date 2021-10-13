use core::{
    cell::{Cell, Ref, RefCell, RefMut},
    hash::Hash,
    iter::FromIterator,
};

type Active = bool;

#[derive(Debug)]
pub struct BattleMap<K: Eq + Hash, V>(hashbrown::HashMap<K, (Cell<Active>, RefCell<V>)>);

impl<K: Eq + Hash, V> BattleMap<K, V> {
    pub fn get(&self, k: &K) -> Option<Ref<V>> {
        self.0
            .get(k)
            .filter(|(b, ..)| b.get())
            .map(|(.., v)| v.try_borrow().ok())
            .flatten()
    }

    pub fn get_mut(&self, k: &K) -> Option<RefMut<V>> {
        self.0
            .get(k)
            .filter(|(b, ..)| b.get())
            .map(|(.., v)| v.try_borrow_mut().ok())
            .flatten()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, Ref<V>)> {
        self.0
            .iter()
            .filter(|(.., (b, ..))| b.get())
            .flat_map(|(k, (.., v))| v.try_borrow().map(|v| (k, v)))
    }

    pub fn iter_mut(&self) -> impl Iterator<Item = (&K, RefMut<V>)> {
        self.0
            .iter()
            .filter(|(.., (b, ..))| b.get())
            .flat_map(|(k, (.., v))| v.try_borrow_mut().map(|v| (k, v)))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = Ref<V>> {
        self.0
            .values()
            .filter(|(b, ..)| b.get())
            .flat_map(|(.., v)| v.try_borrow())
    }

    pub fn values_mut(&self) -> impl Iterator<Item = RefMut<V>> {
        self.0
            .values()
            .filter(|(b, ..)| b.get())
            .flat_map(|(.., v)| v.try_borrow_mut())
    }

    pub fn all_values_mut(&self) -> impl Iterator<Item = RefMut<V>> {
        self.0.values().flat_map(|(.., v)| v.try_borrow_mut())
    }

    pub fn deactivate(&self, k: &K) -> Option<RefMut<V>> {
        if let Some((active, v)) = self.0.get(k) {
            active.set(false);
            v.try_borrow_mut().ok()
        } else {
            None
        }
    }

    // pub fn inactives(&self)

    // pub fn inactives_mut(&self)

    pub fn clear(&mut self) {}
}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for BattleMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(FromIterator::from_iter(
            iter.into_iter().map(|(k, v)| (k, (Cell::new(true), RefCell::new(v)))),
        ))
    }
}
