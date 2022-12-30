use serde::{Deserialize, Serialize};

use battle::pokedex::{ailment::Ailment, pokemon::Health};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ItemUsage {
    #[serde(default)]
    pub conditions: Vec<ItemCondition>,
    pub execute: ItemExecution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ItemExecution {
    Actions(Vec<ItemAction>),
    // Script,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ItemCondition {
    Fainted,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ItemAction {
    CurePokemon(Option<Ailment>),
    HealPokemon(Health),
}

impl Default for ItemExecution {
    fn default() -> Self {
        Self::None
    }
}
