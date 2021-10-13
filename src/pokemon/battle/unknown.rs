use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    pokemon::{owned::OwnedPokemon, Gender, Level, Pokemon, PokemonId},
    Dex, Initializable, Uninitializable,
};

pub type UninitUnknownPokemon = UnknownPokemon<PokemonId>;
pub type InitUnknownPokemon<'d> = UnknownPokemon<&'d Pokemon>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnknownPokemon<P> {
    pub pokemon: P,
    pub nickname: Option<String>,
    pub level: Level,
    pub gender: Option<Gender>,
    pub hp: f32,
    pub ailment: Option<LiveAilment>,
}

impl<'d> InitUnknownPokemon<'d> {
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

    pub fn name<'b: 'd>(&'b self) -> &'b str {
        self.nickname.as_ref().unwrap_or(&self.pokemon.name)
    }

}

impl<'d> Uninitializable for InitUnknownPokemon<'d> {

    type Output = UninitUnknownPokemon;

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

impl<'d, D: Dex<Pokemon> + 'd> Initializable<'d, D> for UninitUnknownPokemon {
    type Output = InitUnknownPokemon<'d>;

    fn init(self, dex: &'d D) -> Option<Self::Output> {
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
