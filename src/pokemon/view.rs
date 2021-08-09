use pokedex::pokemon::InitPokemon;

use super::{battle::BattlePokemon, UnknownPokemon};

pub trait PokemonView {
    fn fainted(&self) -> bool;

    /// Check if hidden (should not be used in battle)
    fn visible(&self) -> bool;
}

impl<'d> PokemonView for BattlePokemon<'d> {
    fn fainted(&self) -> bool {
        InitPokemon::fainted(self)
    }

    fn visible(&self) -> bool {
        !self.caught
    }
}

impl<'d> PokemonView for InitPokemon<'d> {
    fn fainted(&self) -> bool {
        InitPokemon::fainted(self)
    }

    fn visible(&self) -> bool {
        true
    }
}

impl<P> PokemonView for Option<UnknownPokemon<P>> {
    fn fainted(&self) -> bool {
        self.as_ref().map(|u| u.fainted()).unwrap_or_default()
    }

    fn visible(&self) -> bool {
        true
    }
}
