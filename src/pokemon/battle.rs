use core::ops::{Deref, DerefMut};

use pokedex::{moves::MoveId, pokemon::PokemonInstance};

use crate::pokemon::UnknownPokemon;

#[derive(Debug, Clone)]
pub struct BattlePokemon {
    pub instance: PokemonInstance,
    pub learnable_moves: Vec<MoveId>,
    // pub persistent: Option<PersistentMove>,
    pub caught: bool,
    pub known: bool,
    pub flinch: bool,
    pub requestable: bool,
}

impl BattlePokemon {
    pub fn know(&mut self) -> Option<UnknownPokemon> {
        (!self.known).then(|| {
            self.known = true;
            UnknownPokemon::new(&self.instance)
        })
    }

    pub fn try_flinch(&mut self) -> bool {
        if self.flinch {
            self.flinch = false;
            true
        } else {
            false
        }
    }
}

impl From<PokemonInstance> for BattlePokemon {
    fn from(instance: PokemonInstance) -> Self {
        Self {
            instance,
            learnable_moves: Vec::new(),
            caught: false,
            known: false,
            flinch: false,
            requestable: false,
        }
    }
}

impl Deref for BattlePokemon {
    type Target = PokemonInstance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl DerefMut for BattlePokemon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}
