use pokedex::pokemon::owned::OwnedPokemon;

use crate::{
    engine::BattlePokemon,
    party::{Active, ActivePokemon},
    pokemon::PokemonInstance,
    select::BattleSelection,
};

#[derive(Debug, Clone, Copy)]
pub struct ActiveBattlePokemon<ID> {
    pub index: usize,
    pub queued_move: Option<BattleSelection<ID>>,
}

impl<ID> ActiveBattlePokemon<ID> {
    pub fn as_usize(this: &[Option<Self>]) -> Active<usize> {
        this.iter()
            .map(|o| o.as_ref().map(ActivePokemon::index))
            .collect()
    }

    pub fn queued(&self) -> bool {
        self.queued_move.is_some()
    }
}
impl<ID> ActivePokemon for ActiveBattlePokemon<ID> {
    fn index(&self) -> usize {
        self.index
    }
}

impl<ID> From<usize> for ActiveBattlePokemon<ID> {
    fn from(index: usize) -> Self {
        Self {
            index,
            queued_move: None,
        }
    }
}

impl<ID> core::fmt::Display for ActiveBattlePokemon<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "#{}, Queued move: {}",
            self.index,
            self.queued_move.is_some()
        )
    }
}

impl PokemonInstance for BattlePokemon {
    // fn id(&self) -> &PokemonId {
    //     &self.pokemon.id
    // }

    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}
