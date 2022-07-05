use core::{hash::Hash, ops::Deref};

use std::error::Error;

use pokedex::{item::Item, moves::Move, pokemon::Pokemon};

use rand::Rng;

use crate::{
    engine::{BattlePokemon, Players},
    pokemon::{Indexed, PokemonIdentifier},
    prelude::BattleData, item::engine::ItemResult, moves::engine::MoveResult,
};

pub trait ScriptingEngine {
    type Error: Error;

    fn execute_move<
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
        R: Rng + Clone + 'static,
        ID: Eq + Hash + Clone + 'static + core::fmt::Debug,
        PLR: Players<ID, P, M, I>,
    >(
        &self,
        random: &mut R,
        m: &Move,
        user: Indexed<ID, &BattlePokemon<P, M, I>>,
        targets: Vec<PokemonIdentifier<ID>>,
        players: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error>;

    fn execute_item<
        ID: PartialEq,
        R: Rng,
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
        PLR: Players<ID, P, M, I>,
    >(
        &mut self,
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
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
        R: Rng + Clone + 'static,
        ID: Eq + Hash + Clone + 'static + core::fmt::Debug,
        PLR: Players<ID, P, M, I>,
    >(
        &self,
        _: &mut R,
        _: &Move,
        _: Indexed<ID, &BattlePokemon<P, M, I>>,
        _: Vec<PokemonIdentifier<ID>>,
        _: &PLR,
    ) -> Result<Vec<Indexed<ID, MoveResult>>, Self::Error> {
        Err(DefaultScriptError)
    }

    fn execute_item<
        ID: PartialEq,
        R: Rng,
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
        PLR: Players<ID, P, M, I>,
    >(
        &mut self,
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
