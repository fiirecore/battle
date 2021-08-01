use serde::{Deserialize, Serialize};

use pokedex::{
    item::ItemRef,
    moves::{usage::Critical, MoveRef},
    pokemon::{stat::StatStage, Experience, Level},
    status::StatusEffectInstance,
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
    Status(StatusEffectInstance),
    Faint(PokemonIndex<ID>), // target that is fainting
    SetExp(Experience, Level),
    Fail,
}

pub type BoundClientMove<ID> = BoundAction<ID, ClientMove<ID>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMove<ID> {
    Move(MoveRef, Vec<ClientActions<ID>>),
    Switch(usize),
    UseItem(ItemRef, usize),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientActions<ID> {
    pub location: MoveTargetLocation,
    pub actions: Vec<ClientAction<ID>>,
}
