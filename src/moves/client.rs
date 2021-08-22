use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    item::ItemId,
    moves::MoveId,
    pokemon::{Experience, Level},
    types::Effective,
};

use crate::{
    moves::usage::{target::MoveTargetLocation, Critical},
    pokemon::{
        battle::stat::{BattleStatType, Stage},
        PokemonIndex,
    },
    BoundAction,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClientAction<ID> {
    Miss,
    TargetHP(f32, Critical), // bool = crit
    UserHP(f32),             // dont heal the target
    Effective(Effective),
    Stat(BattleStatType, Stage),
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
