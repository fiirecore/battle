use core::ops::Deref;

use rand::Rng;

use pokedex::{item::Item, moves::Move, pokemon::Pokemon};

use crate::{
    data::BattleData, default_engine::scripting::ScriptingEngine, engine::Players,
    item::engine::ItemResult, pokemon::PokemonIdentifier,
};

use super::ItemError;

impl ScriptingEngine {
    pub fn execute_item<
        ID: PartialEq,
        R: Rng,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
        PLR: Players<ID, P, M, I>,
    >(
        &mut self,
        _battle: &BattleData,
        _random: &mut R,
        _item: &Item,
        _user: &ID,
        _target: PokemonIdentifier<ID>,
        _players: &mut PLR,
    ) -> Result<Vec<ItemResult>, ItemError> {
        log::debug!("to - do: item scripting");
        Ok(vec![])
    }
}
