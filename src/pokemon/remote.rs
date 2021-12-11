use core::ops::Deref;

use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    item::Item,
    moves::Move,
    pokemon::{data::Gender, owned::OwnedPokemon, Level, Pokemon, PokemonId},
    Dex, Initializable, Uninitializable,
};

pub type RemotePokemon = UnknownPokemon<PokemonId>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnknownPokemon<P> {
    pub pokemon: P,
    pub nickname: Option<String>,
    pub level: Level,
    pub gender: Gender,
    pub hp: f32,
    pub ailment: Option<LiveAilment>,
}

impl<P> UnknownPokemon<P> {
    pub fn fainted(&self) -> bool {
        self.hp <= 0.0
    }
}

impl<P: Deref<Target = Pokemon>> UnknownPokemon<P> {
    pub fn new<M: Deref<Target = Move>, I: Deref<Target = Item>>(
        pokemon: &OwnedPokemon<P, M, I>,
    ) -> Self
    where
        P: Clone,
    {
        Self {
            pokemon: pokemon.pokemon.clone(),
            nickname: pokemon.nickname.clone(),
            level: pokemon.level,
            gender: pokemon.gender,
            hp: pokemon.percent_hp(),
            ailment: pokemon.ailment,
        }
    }

    pub fn name(&self) -> &str {
        self.nickname.as_ref().unwrap_or(&self.pokemon.name)
    }
}

impl<P: Deref<Target = Pokemon>> Uninitializable for UnknownPokemon<P> {
    type Output = RemotePokemon;

    fn uninit(self) -> Self::Output {
        Self::Output {
            pokemon: self.pokemon.id,
            nickname: self.nickname,
            level: self.level,
            gender: self.gender,
            hp: self.hp,
            ailment: self.ailment,
        }
    }
}

impl<'d, P: Deref<Target = Pokemon>> Initializable<'d, Pokemon, P> for RemotePokemon {
    type Output = UnknownPokemon<P>;

    fn init(self, dex: &'d dyn Dex<'d, Pokemon, P>) -> Option<Self::Output> {
        Some(Self::Output {
            pokemon: dex.try_get(&self.pokemon)?,
            nickname: self.nickname,
            level: self.level,
            gender: self.gender,
            hp: self.hp,
            ailment: self.ailment,
        })
    }
}
