use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use rand::Rng;
use rhai::INT;

use pokedex::{
    moves::MoveCategory,
    types::{Effective, PokemonType},
};

use crate::{
    moves::usage::DamageResult,
    pokemon::{battle::BattlePokemon, OwnedRefPokemon},
};

use super::{ScriptDamage, ScriptRandom};

#[derive(Clone, Copy)]
pub struct ScriptPokemon<R: Rng + Clone + 'static>(*const BattlePokemon<'static>, PhantomData<R>);

impl<R: Rng + Clone + 'static> ScriptPokemon<R> {
    pub fn new<'a>(pokemon: &BattlePokemon<'a>) -> Self {
        let p = pokemon as *const BattlePokemon<'a>;
        let p = unsafe {
            core::mem::transmute::<*const BattlePokemon<'a>, *const BattlePokemon<'static>>(p)
        };
        Self(p, PhantomData)
    }

    pub fn get_damage(
        &mut self,
        random: ScriptRandom<R>,
        target: ScriptPokemon<R>,
        power: INT,
        category: MoveCategory,
        move_type: PokemonType,
        crit_rate: INT,
    ) -> ScriptDamage {
        let mut random = random;
        self.move_power_damage_random(
            random.deref_mut(),
            &target,
            power as _,
            category,
            move_type,
            crit_rate as _,
        )
        .map(ScriptDamage::from)
        .unwrap_or(
            DamageResult {
                damage: 0,
                effective: Effective::Ineffective,
                crit: false,
            }
            .into(),
        )
    }
    pub fn hp(&mut self) -> INT {
        OwnedRefPokemon::hp(self) as INT
    }
}

impl<R: Rng + Clone + 'static> Deref for ScriptPokemon<R> {
    type Target = BattlePokemon<'static>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
