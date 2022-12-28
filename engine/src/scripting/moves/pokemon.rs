use core::ops::{Deref, DerefMut};

use rand::Rng;
use rhai::INT;

use battle::{
    pokedex::types::PokemonType,
    engine::pokemon::{crit, throw_move, BattlePokemon},
    moves::MoveCategory,
    pokemon::{Indexed, TeamIndex},
};

use super::{ScriptDamage, ScriptMove, ScriptRandom};

#[derive(Debug)]
pub struct DerefPtr<T>(*const T);

impl<T> DerefPtr<T> {
    pub fn of(t: &T) -> Self {
        Self(t as *const T)
    }
}

impl<T> Deref for DerefPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> From<&T> for DerefPtr<T> {
    fn from(t: &T) -> Self {
        Self(t as _)
    }
}

impl<T> Clone for DerefPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for DerefPtr<T> {}

#[derive(Debug, Clone)]
pub struct ScriptPokemon<ID: Clone + Send + Sync + 'static>(TeamIndex<ID>, *mut BattlePokemon);

impl<ID: Clone + Send + Sync + 'static> ScriptPokemon<ID> {
    pub fn new(pokemon: Indexed<ID, &mut BattlePokemon>) -> Self {
        let Indexed(id, pokemon) = pokemon;
        Self(id, pokemon as _)
    }

    pub fn position(&self) -> &TeamIndex<ID> {
        &self.0
    }

    pub fn throw_move<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        mut random: ScriptRandom<R>,
        m: ScriptMove,
    ) -> bool {
        throw_move(random.deref_mut(), m.accuracy)
    }

    // pub fn ailment_affects(
    //     &mut self,
    //     ailment: ScriptAilmentEffect,
    // ) -> bool {
    //     true
    // }

    pub fn get_damage<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        random: ScriptRandom<R>,
        target: Self,
        power: INT,
        category: MoveCategory,
        move_type: PokemonType,
        crit_rate: INT,
    ) -> ScriptDamage {
        let mut random = random;
        let crit = crit(random.deref_mut(), crit_rate as _);
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
        self.hp as _
    }
}

unsafe impl<ID: Clone + Send + Sync + 'static> Send for ScriptPokemon<ID> {}
unsafe impl<ID: Clone + Send + Sync + 'static> Sync for ScriptPokemon<ID> {}

impl<ID: Clone + Send + Sync + 'static> Deref for ScriptPokemon<ID> {
    type Target = BattlePokemon;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.1 }
    }
}

impl<ID: Clone + Send + Sync + 'static> DerefMut for ScriptPokemon<ID> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.1 }
    }
}

impl<ID: Clone + Send + Sync + 'static> Into<TeamIndex<ID>> for ScriptPokemon<ID> {
    fn into(self) -> TeamIndex<ID> {
        self.0
    }
}
