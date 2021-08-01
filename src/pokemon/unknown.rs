use serde::{Deserialize, Serialize};

use pokedex::{
    pokemon::{data::Gender, Level, PokemonInstance, PokemonRef},
    status::StatusEffectInstance,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnknownPokemon {
    pub pokemon: PokemonRef,
    pub nickname: Option<String>,
    pub level: Level,
    pub gender: Gender,
    pub hp: f32,
    pub effect: Option<StatusEffectInstance>,
    pub instance: Option<PokemonInstance>,
}

impl UnknownPokemon {
    pub fn new(pokemon: &PokemonInstance) -> Self {
        Self {
            pokemon: pokemon.pokemon,
            nickname: pokemon.nickname.clone(),
            level: pokemon.level,
            gender: pokemon.gender,
            hp: pokemon.percent_hp(),
            effect: pokemon.effect,
            instance: None,
        }
    }

    pub fn fainted(&self) -> bool {
        self.hp <= 0.0
    }

}

impl UnknownPokemon {
    pub fn name(&self) -> &str {
        self.nickname.as_ref().unwrap_or(&self.pokemon.name)
    }
}
