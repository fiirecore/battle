use core::{
    cell::{Cell, Ref, RefCell, RefMut},
    hash::Hash,
    iter::FromIterator,
};

pub struct BattleMap<K: Eq + Hash, V>(hashbrown::HashMap<K, (Properties, RefCell<V>)>);

pub struct Properties {
    pub active: Cell<bool>,
    pub waiting: Cell<bool>,
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

    pub fn len(&self) -> usize {
        self.0.len()
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
        self.0.keys()
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

    pub fn values_waiting_mut(&self) -> impl Iterator<Item = (&Cell<bool>, RefMut<V>)> {
        self.0.values().filter(|(p, ..)| p.active.get()).flat_map(|(p, v)| v.try_borrow_mut().map(|v| (&p.waiting, v)))
    }

    pub fn all_waiting(&self) -> bool {
        self.0.values().all(|(p, ..)| p.waiting.get())
    }

    // pub fn inactives(&self)

    // pub fn inactives_mut(&self)

    pub fn clear(&mut self) {}
}

impl Default for Properties {
    fn default() -> Self {
        Self { active: Cell::new(true), waiting: Default::default() }
    }
}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for BattleMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(FromIterator::from_iter(
            iter.into_iter().map(|(k, v)| (k, (Default::default(), RefCell::new(v)))),
        ))
    }
}
