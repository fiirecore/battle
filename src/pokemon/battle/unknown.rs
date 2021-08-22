use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::Ailment,
    pokemon::{Gender, Level, PokemonId, PokemonRef, Pokedex},
};

use super::OwnedRefPokemon;

pub type UninitUnknownPokemon = UnknownPokemon<PokemonId>;
pub type InitUnknownPokemon<'d> = UnknownPokemon<PokemonRef<'d>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnknownPokemon<P> {
    pub pokemon: P,
    pub nickname: Option<String>,
    pub level: Level,
    pub gender: Option<Gender>,
    pub hp: f32,
    pub ailment: Option<Ailment>,
}

impl<'d> InitUnknownPokemon<'d> {
    pub fn new(pokemon: &OwnedRefPokemon<'d>) -> Self {
        Self {
            pokemon: pokemon.pokemon,
            nickname: pokemon.nickname.clone(),
            level: pokemon.level,
            gender: pokemon.gender,
            hp: pokemon.percent_hp(),
            ailment: pokemon.ailment.as_ref().map(|a| a.ailment),
        }
    }

    pub fn name<'b: 'd>(&'b self) -> &'b str {
        self.nickname.as_ref().unwrap_or(&self.pokemon.name)
    }

    pub fn uninit(self) -> UninitUnknownPokemon {
        UninitUnknownPokemon {
            pokemon: self.pokemon.id,
            nickname: self.nickname,
            level: self.level,
            gender: self.gender,
            hp: self.hp,
            ailment: self.ailment,
        }
    }

}

impl UninitUnknownPokemon {

    pub fn init<'d>(self, pokedex: &'d Pokedex) -> Option<InitUnknownPokemon<'d>> {
        Some(InitUnknownPokemon {
            pokemon: pokedex.try_get(&self.pokemon)?,
            nickname: self.nickname,
            level: self.level,
            gender: self.gender,
            hp: self.hp,
            ailment: self.ailment,
        })
    }

}

impl<P> UnknownPokemon<P> {
    pub fn fainted(&self) -> bool {
        self.hp <= 0.0
    }
}
