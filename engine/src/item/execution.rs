use serde::{Deserialize, Serialize};

use battle::{pokedex::{ailment::Ailment, pokemon::Health, item::usage::ItemExecution}};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum BattleItemExecution {
    Normal(ItemExecution),
    Script,
    Pokeball,
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

impl Default for BattleItemExecution {
    fn default() -> Self {
        Self::Normal(ItemExecution::None)
    }
}
