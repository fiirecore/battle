use serde::{Serialize, de::DeserializeOwned};

use crate::prelude::{BattleData, PlayerData};

use super::BattleState;

pub struct SerializedBattle<ID: Serialize + DeserializeOwned> {
    pub state: BattleState<ID>,
    pub data: BattleData,
    pub players: hashbrown::HashMap<ID, PlayerData<>>,
}