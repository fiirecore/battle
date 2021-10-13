use core::fmt::Display;
use serde::{Deserialize, Serialize};

use pokedex::pokemon::owned::OwnedPokemon;

use crate::{moves::BattleMove, party::PartyIndex};

pub mod battle;

pub type ActivePosition = usize;
pub type PartyPosition = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PokemonIndex<ID> {
    pub team: ID,
    pub index: usize,
}

impl<ID: Sized + Display> Display for PokemonIndex<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} #{}", self.team, self.index)
    }
}

pub trait PokemonView {
    fn fainted(&self) -> bool;
}

impl<'d> PokemonView for battle::BattlePokemon<'d> {
    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}

impl<'d> PokemonView for OwnedPokemon<'d> {
    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}

impl<P> PokemonView for Option<battle::UnknownPokemon<P>> {
    fn fainted(&self) -> bool {
        self.as_ref().map(|u| u.fainted()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActivePokemon {
    pub index: usize,
    pub queued_move: Option<BattleMove>,
}

impl PartyIndex for ActivePokemon {
    fn index(&self) -> usize {
        self.index
    }
}

impl From<usize> for ActivePokemon {
    fn from(index: usize) -> Self {
        Self {
            index,
            queued_move: None,
        }
    }
}
