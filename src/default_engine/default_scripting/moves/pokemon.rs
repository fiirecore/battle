use core::ops::{Deref, DerefMut};

use rand::Rng;
use rhai::INT;

use pokedex::{moves::MoveCategory, pokemon::owned::OwnedPokemon, types::PokemonType};

use crate::{
    engine::pokemon::{crit, throw_move, BattlePokemon},
    pokemon::{Indexed, PokemonIdentifier},
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
pub struct ScriptPokemon<ID: Clone>(PokemonIdentifier<ID>, pub BattlePokemon);

impl<ID: Clone> ScriptPokemon<ID> {

    pub fn new(pokemon: Indexed<ID, &BattlePokemon>) -> Self {
        let Indexed(id, pokemon) = pokemon;
        let p = BattlePokemon {
            p: OwnedPokemon {
                pokemon: pokemon.pokemon.clone(),
                level: pokemon.level,
                gender: pokemon.gender,
                nature: pokemon.nature,
                hp: pokemon.hp,
                ivs: pokemon.ivs,
                evs: pokemon.evs,
                friendship: pokemon.friendship,
                ailment: pokemon.ailment,
                nickname: None,
                moves: Default::default(),
                item: pokemon.item.clone(),
                experience: pokemon.experience,
            },
            stages: pokemon.stages.clone(),
        };

        Self(id, p)
    }

    pub fn throw_move<R: Rng + Clone + 'static>(
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

impl<ID: Clone> Deref for ScriptPokemon<ID> {
    type Target = BattlePokemon;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<ID: Clone> Into<PokemonIdentifier<ID>> for ScriptPokemon<ID> {
    fn into(self) -> PokemonIdentifier<ID> {
        self.0
    }
}
