use core::ops::{Deref, DerefMut};

use rhai::INT;
use rand::Rng;

use pokedex::{moves::MoveCategory, types::PokemonType};

use pokedex::{item::Item, moves::Move, pokemon::owned::OwnablePokemon, pokemon::Pokemon};

use crate::{
    engine::pokemon::{crit, throw_move, BattlePokemon},
    pokemon::{Indexed, PokemonIdentifier},
};

use super::{moves::ScriptMove, ScriptDamage, ScriptRandom};

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

type Pkmn = BattlePokemon<DerefPtr<Pokemon>, DerefPtr<Move>, DerefPtr<Item>>;

#[derive(Debug, Clone)]
pub struct ScriptPokemon<ID: Clone>(PokemonIdentifier<ID>, pub Pkmn);

impl<ID: Clone> ScriptPokemon<ID> {
    // pub fn from_player<'d, const AS: usize>(
    //     (id, p): (PokemonIdentifier<ID>, Ref<BattlePlayer<'d, ID, AS>>),
    // ) -> Option<Self> {
    //     p.party
    //         .active(id.index())
    //         .map(|p| Self::new(Indexed(id, p)))
    // }

    pub fn new<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>>(
        pokemon: Indexed<ID, &BattlePokemon<P, M, I>>,
    ) -> Self {
        let Indexed(id, pokemon) = pokemon;
        let p = BattlePokemon {
            p: OwnablePokemon {
                pokemon: DerefPtr(pokemon.pokemon.deref() as _),
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
                item: pokemon.item.as_ref().map(|i| DerefPtr::of(i.deref())),
                experience: pokemon.experience,
            },
            stages: pokemon.stages.clone(),
        };

        // let p = pokemon.1 as *const BattlePokemon<'d>;
        // let p = unsafe {
        //     core::mem::transmute::<*const BattlePokemon<'d>, *const BattlePokemon<'static>>(p)
        // };
        Self(id, p)
    }

    pub fn throw_move<R: Rng + Clone + 'static>(
        &mut self,
        mut random: ScriptRandom<R>,
        m: ScriptMove,
    ) -> bool {
        throw_move(random.deref_mut(), m.accuracy)
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
    type Target = Pkmn;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<ID: Clone> Into<PokemonIdentifier<ID>> for ScriptPokemon<ID> {
    fn into(self) -> PokemonIdentifier<ID> {
        self.0
    }
}
