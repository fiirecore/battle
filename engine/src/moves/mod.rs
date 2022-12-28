use battle::moves::BattleMove;

mod execution;
pub use execution::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMove {
    pub data: BattleMove,
    pub usage: MoveExecution,
}