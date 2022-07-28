use alloc::sync::Arc;

use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    pokemon::{data::Gender, owned::OwnedPokemon, Level, Pokemon, PokemonId},
    Dex,
};

pub type RemotePokemon = UnknownPokemon<PokemonId>;
pub type InitUnknownPokemon = UnknownPokemon<Arc<Pokemon>>;

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

impl InitUnknownPokemon {
    pub fn new(pokemon: &OwnedPokemon) -> Self {
        Self {
            pokemon: pokemon.pokemon.clone(),
            nickname: pokemon.nickname.clone(),
            level: pokemon.level,
            gender: pokemon.gender,
            hp: pokemon.percent_hp(),
            ailment: pokemon.ailment,
        }
    }

    pub fn oname(u: Option<&Self>) -> &str {
        u.map(UnknownPokemon::name).unwrap_or("Unknown")
    }

    pub fn name(&self) -> &str {
        self.nickname.as_ref().unwrap_or(&self.pokemon.name)
    }
}

impl InitUnknownPokemon {
    pub fn uninit(self) -> RemotePokemon {
        RemotePokemon {
            pokemon: self.pokemon.id,
            nickname: self.nickname,
            level: self.level,
            gender: self.gender,
            hp: self.hp,
            ailment: self.ailment,
        }
    }
}

impl RemotePokemon {
    pub fn init(self, dex: &Dex<Pokemon>) -> Option<InitUnknownPokemon> {
        Some(UnknownPokemon {
            pokemon: dex.try_get(&self.pokemon)?.clone(),
            nickname: self.nickname,
            level: self.level,
            gender: self.gender,
            hp: self.hp,
            ailment: self.ailment,
        })
    }
}
