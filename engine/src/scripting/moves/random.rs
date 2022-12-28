use core::ops::{Deref, DerefMut};

use rand::Rng;
use rhai::INT;

#[derive(Clone, Copy)]
pub struct ScriptRandom<R: Rng + Clone + Send + Sync + 'static>(*mut R);

unsafe impl<R: Rng + Clone + Send + Sync + 'static> Send for ScriptRandom<R> {}

unsafe impl<R: Rng + Clone + Send + Sync + 'static> Sync for ScriptRandom<R> {}

impl<R: Rng + Clone + Send + Sync + 'static> ScriptRandom<R> {
    pub fn new(random: &mut R) -> Self {
        Self(random as _)
    }

    pub fn chance(&mut self, percent: INT) -> bool {
        self.deref_mut().gen_bool(percent as f64 / 100.0)
    }
}

impl<R: Rng + Clone + Send + Sync + 'static> Deref for ScriptRandom<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<R: Rng + Clone + Send + Sync + 'static> DerefMut for ScriptRandom<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
