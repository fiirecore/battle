use core::ops::{Deref, DerefMut};

use rand::Rng;
use rhai::INT;

use pokedex::{moves::MoveCategory, pokemon::owned::OwnedPokemon, types::PokemonType};

use crate::{moves::target::TargetLocation, pokemon::battle::BattlePokemon};

use super::{moves::ScriptMove, ScriptDamage, ScriptRandom};

#[derive(Clone, Copy)]
pub struct ScriptPokemon(
    TargetLocation,
    *const BattlePokemon<'static>,
    // PhantomData<R>,
);

impl ScriptPokemon {
    pub fn new<'a>(pokemon: (TargetLocation, &BattlePokemon<'a>)) -> Self {
        let p = pokemon.1 as *const BattlePokemon<'a>;
        let p = unsafe {
            core::mem::transmute::<*const BattlePokemon<'a>, *const BattlePokemon<'static>>(p)
        };
        Self(pokemon.0, p)
    }

    pub fn throw_move<R: Rng + Clone + 'static>(
        &mut self,
        random: ScriptRandom<R>,
        m: ScriptMove,
    ) -> bool {
        let mut random = random;
        BattlePokemon::throw_move(self, random.deref_mut(), m.m())
    }

    pub fn get_damage<R: Rng + Clone + 'static>(
        &mut self,
        random: ScriptRandom<R>,
        target: Self,
        power: INT,
        category: MoveCategory,
        move_type: PokemonType,
        crit_rate: INT,
    ) -> ScriptDamage {
        let mut random = random;
        let crit = BattlePokemon::crit(random.deref_mut(), crit_rate as _);
        ScriptDamage::from(self.move_power_damage_random(
            random.deref_mut(),
            &target,
            power as _,
            category,
            move_type,
            crit,
        ))
    }
    pub fn hp(&mut self) -> INT {
        OwnedPokemon::hp(self) as INT
    }
}

impl Deref for ScriptPokemon {
    type Target = BattlePokemon<'static>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.1 }
    }
}

impl Into<TargetLocation> for ScriptPokemon {
    fn into(self) -> TargetLocation {
        self.0
    }
}
