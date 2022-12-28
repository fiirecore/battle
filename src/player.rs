use std::sync::Arc;

use serde::{Deserialize, Serialize};

use pokedex::{
    item::bag::{OwnedBag},
};

use crate::{
    party::{ActivePokemon, PlayerParty},
    pokemon::PokemonInstance, endpoint::BattleEndpoint, 
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PlayerSettings {
    pub gains_exp: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self { gains_exp: true }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum RemovalReason {
    Loss,
    Run,
}