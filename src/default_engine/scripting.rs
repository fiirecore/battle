use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash};
use std::error::Error;

// use std::error::Error;

use pokedex::{item::Item, moves::Move};

use rand::Rng;

use crate::{
    engine::{BattlePokemon, Players},
    engine::{ItemResult, MoveResult},
    pokemon::{Indexed, PokemonIdentifier},
    prelude::BattleData,
};

pub trait ScriptingEngine {
    type Error: Error;

    fn execute_move<
        ID: Eq + Hash + Clone + 'static + Debug,
        R: Rng + Clone + 'static,
        PLR: Players<ID>,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon>,
        targets: Vec<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error>;

    fn execute_item<ID: PartialEq, R: Rng, PLR: Players<ID>>(
        &self,
        battle: &BattleData,
        random: &mut R,
        item: &Item,
        user: &ID,
        target: PokemonIdentifier<ID>,
        players: &mut PLR,
    ) -> Result<Vec<ItemResult>, Self::Error>;
}

pub struct DefaultScriptEngine;

impl ScriptingEngine for DefaultScriptEngine {
    type Error = DefaultScriptError;

    fn execute_move<
        ID: Eq + Hash + Clone + 'static + core::fmt::Debug,
        R: Rng + Clone + 'static,
        PLR: Players<ID>,
    >(
        &self,
        _: &mut R,
        _: &Move,
        _: Indexed<ID, &BattlePokemon>,
        _: Vec<PokemonIdentifier<ID>>,
        _: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error> {
        Err(DefaultScriptError)
    }

    fn execute_item<ID: PartialEq, R: Rng, PLR: Players<ID>>(
        &self,
        _: &BattleData,
        _: &mut R,
        _: &Item,
        _: &ID,
        _: PokemonIdentifier<ID>,
        _: &mut PLR,
    ) -> Result<Vec<ItemResult>, Self::Error> {
        Err(DefaultScriptError)
    }
}

#[derive(Debug)]
pub struct DefaultScriptError;

impl Error for DefaultScriptError {}

impl core::fmt::Display for DefaultScriptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "No script engine is in use!")
    }
}
