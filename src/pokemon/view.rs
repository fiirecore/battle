use super::{battle::BattlePokemon, UnknownPokemon, OwnedRefPokemon};

pub trait PokemonView {
    fn fainted(&self) -> bool;
}

impl<'d> PokemonView for BattlePokemon<'d> {
    fn fainted(&self) -> bool {
        OwnedRefPokemon::fainted(self)
    }

}

impl<'d> PokemonView for OwnedRefPokemon<'d> {
    fn fainted(&self) -> bool {
        OwnedRefPokemon::fainted(self)
    }

}

impl<P> PokemonView for Option<UnknownPokemon<P>> {
    fn fainted(&self) -> bool {
        self.as_ref().map(|u| u.fainted()).unwrap_or_default()
    }
}
