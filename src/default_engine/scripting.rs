use hashbrown::HashMap;

use rand::Rng;
use quad_compat_rhai::{
    packages::{BasicArrayPackage, Package},
    Engine,
};

use pokedex::{item::ItemId, moves::MoveId};

use crate::moves::engine::MoveResult;

use super::moves::scripting::*;

type Scripts<ID> = HashMap<ID, String>;

pub type MoveScripts = Scripts<MoveId>;
pub type ItemScripts = Scripts<ItemId>;

pub struct ScriptingEngine {
    pub engine: Engine,
    pub moves: MoveScripts,
    pub items: ItemScripts,
}

impl ScriptingEngine {
    pub fn new<ID: Clone + 'static, R: Rng + Clone + 'static>() -> Self {
        let mut engine = Engine::new_raw();

        

        engine
            .register_global_module(BasicArrayPackage::new().as_shared_module())
            .register_type_with_name::<ScriptRandom<R>>("Random")
            .register_type_with_name::<ScriptDamage>("Damage")
            .register_fn("damage", ScriptDamage::with_damage)
            .register_set("damage", ScriptDamage::set_damage)
            .register_get("damage", ScriptDamage::get_damage)
            .register_get("effective", ScriptDamage::effective)
            .register_iterator::<Vec<ScriptPokemon<ID>>>()
            .register_type_with_name::<ScriptPokemon<ID>>("Pokemon")
            .register_fn("throw_move", ScriptPokemon::<ID>::throw_move::<R>)
            .register_fn("damage", ScriptPokemon::<ID>::get_damage::<R>)
            .register_get("hp", ScriptPokemon::<ID>::hp)
            .register_iterator::<Vec<ScriptPokemon<ID>>>()
            .register_type_with_name::<ScriptMove>("Move")
            .register_get("category", ScriptMove::get_category)
            .register_get("type", ScriptMove::get_type)
            .register_get("crit_rate", ScriptMove::get_crit_rate)
            .register_type_with_name::<MoveCategory>("Category")
            .register_type_with_name::<PokemonType>("Type")
            .register_type::<MoveResult>()
            .register_type_with_name::<ScriptMoveResult<ID>>("Result")
            .register_fn("Miss", ScriptMoveResult::<ID>::miss)
            .register_fn("Damage", ScriptMoveResult::<ID>::damage)
            .register_fn("Drain", ScriptMoveResult::<ID>::heal);

        Self {
            items: Default::default(),
            moves: Default::default(),
            engine,
        }
    }
}
