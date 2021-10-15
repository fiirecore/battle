use core::ops::{Deref, DerefMut};

use rand::Rng;

#[derive(Clone, Copy)]
pub struct ScriptRandom<R: Rng>(*mut R);

impl<R: Rng> ScriptRandom<R> {
    pub fn new(random: &mut R) -> Self {
        Self(random as _)
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