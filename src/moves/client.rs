use serde::{Deserialize, Serialize};

use pokedex::{
    item::ItemId,
    moves::{usage::Critical, MoveId},
    pokemon::{stat::StatStage, Experience, Level},
    ailment::LiveAilment,
    types::Effective,
};

use crate::{pokemon::PokemonIndex, BoundAction, moves::MoveTargetLocation};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClientAction<ID> {
    Miss,
    TargetHP(f32, Critical), // bool = crit
    UserHP(f32),             // dont heal the target
    Effective(Effective),
    StatStage(StatStage),
    Ailment(LiveAilment),
    Faint(PokemonIndex<ID>), // target that is fainting
    SetExp(Experience, Level),
    Fail,
}

pub type BoundClientMove<ID> = BoundAction<ID, ClientMove<ID>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMove<ID> {
    Move(MoveId, Vec<ClientActions<ID>>),
    Switch(usize),
    UseItem(ItemId, usize),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientActions<ID> {
    pub location: MoveTargetLocation,
    pub actions: Vec<ClientAction<ID>>,
}
