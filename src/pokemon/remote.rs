use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    pokemon::{owned::OwnedPokemon, Gender, Level, Pokemon, PokemonId},
    Dex, Initializable, Uninitializable,
};

pub type RemotePokemon = UnknownPokemon<PokemonId>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnknownPokemon<P> {
    pub pokemon: P,
    pub nickname: Option<String>,
    pub level: Level,
    pub gender: Option<Gender>,
    pub hp: f32,
    pub ailment: Option<LiveAilment>,
}

impl<'d> UnknownPokemon<&'d Pokemon> {
    pub fn new(pokemon: &OwnedPokemon<'d>) -> Self {
        Self {
            pokemon: pokemon.pokemon,
            nickname: pokemon.nickname.clone(),
            level: pokemon.level,
            gender: pokemon.gender,
            hp: pokemon.percent_hp(),
            ailment: pokemon.ailment,
        }
    }

}

impl<'d> UnknownPokemon<&'d Pokemon> {

    pub fn name<'b: 'd>(&'b self) -> &'b str {
        self.nickname.as_ref().unwrap_or(&self.pokemon.name)
    }

}

impl<'d> Uninitializable for UnknownPokemon<&'d Pokemon> {

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

impl<'d> Initializable<'d, Pokemon> for RemotePokemon {
    type Output = UnknownPokemon<&'d Pokemon>;

    fn init(self, dex: &'d dyn Dex<Pokemon>) -> Option<Self::Output> {
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

impl<P> UnknownPokemon<P> {
    pub fn fainted(&self) -> bool {
        self.hp <= 0.0
    }
}
