use serde::{Deserialize, Serialize};
use crate::moves::usage::MoveUsage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentMove {
    pub length: Option<(u8, u8)>, // min,max
    pub actions: MoveUsage,
    pub same_move: bool,
    // pub secondary: Option<MoveActionType>,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentMoveInstance {
    pub actions: MoveUsage,
    pub remaining: Option<u8>,
    pub same_move: bool, // what does this bool mean?
}