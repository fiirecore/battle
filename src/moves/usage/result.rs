use pokedex::{ailment::LiveAilment, pokemon::Health};

use crate::{
    moves::usage::DamageResult,
    pokemon::battle::stat::{BattleStatType, Stage},
};

/// To - do: MoveResults for user, and other targets
pub type MoveResults = Vec<MoveResult>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Damage(DamageResult<Health>),
    Ailment(LiveAilment),
    Drain(DamageResult<Health>, i16), // damage, health gained/lost
    Stat(BattleStatType, Stage),
    Flinch,
    NoHit(NoHitResult),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoHitResult {
    Ineffective,
    Miss,
    Todo,
    Error,
}
