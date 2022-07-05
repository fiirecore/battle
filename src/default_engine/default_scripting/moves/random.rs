use core::ops::{Deref, DerefMut};

use rand::Rng;
use rhai::INT;

#[derive(Clone, Copy)]
pub struct ScriptRandom<R: Rng>(*mut R);

impl<R: Rng> ScriptRandom<R> {
    pub fn new(random: &mut R) -> Self {
        Self(random as _)
    }

    pub fn chance(&mut self, percent: INT) -> bool {
        self.deref_mut().gen_bool(percent as f64 / 100.0)
    }

}

impl<R: Rng> Deref for ScriptRandom<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<R: Rng> DerefMut for ScriptRandom<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
