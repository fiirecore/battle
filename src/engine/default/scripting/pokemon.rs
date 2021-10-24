use core::ops::{Deref, DerefMut};

use rand::Rng;
use rhai::INT;

use pokedex::{moves::MoveCategory, pokemon::owned::OwnedPokemon, types::PokemonType};

use crate::{
    engine::BattlePokemon,
    pokemon::{Indexed, PokemonIdentifier},
};

use super::{moves::ScriptMove, ScriptDamage, ScriptRandom};

#[derive(Clone, Copy)]
pub struct ScriptPokemon<ID>(Indexed<ID, *const BattlePokemon<'static>>);

impl<ID> ScriptPokemon<ID> {
    // pub fn from_player<'d, const AS: usize>(
    //     (id, p): (PokemonIdentifier<ID>, Ref<BattlePlayer<'d, ID, AS>>),
    // ) -> Option<Self> {
    //     p.party
    //         .active(id.index())
    //         .map(|p| Self::new(Indexed(id, p)))
    // }

    pub fn new<'d>(pokemon: Indexed<ID, &BattlePokemon<'d>>) -> Self {
        let p = pokemon.1 as *const BattlePokemon<'d>;
        let p = unsafe {
            core::mem::transmute::<*const BattlePokemon<'d>, *const BattlePokemon<'static>>(p)
        };
        Self(Indexed(pokemon.0, p))
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

impl<ID> Deref for ScriptPokemon<ID> {
    type Target = BattlePokemon<'static>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 .1 }
    }
}

impl<ID> Into<PokemonIdentifier<ID>> for ScriptPokemon<ID> {
    fn into(self) -> PokemonIdentifier<ID> {
        self.0 .0
    }
}
