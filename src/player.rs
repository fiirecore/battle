use pokedex::pokemon::PokemonParty;

use crate::{
    party::{BattleParty, PlayerParty},
    pokemon::{BattlePokemon, UnknownPokemon},
    BattleEndpoint,
};

mod settings;
pub use settings::*;

mod knowable;
pub use knowable::*;

mod validate;
pub use validate::*;

#[cfg(feature = "ai")]
pub mod ai;

pub struct BattlePlayer<ID> {
    pub endpoint: Box<dyn BattleEndpoint<ID>>,
    pub party: BattleParty<ID>,
    pub name: Option<String>,
    pub settings: PlayerSettings,
    /// Player's turn has finished
    pub waiting: bool,
}

impl<ID> BattlePlayer<ID> {
    pub fn new(
        id: ID,
        party: PokemonParty,
        name: Option<String>,
        settings: PlayerSettings,
        endpoint: Box<dyn BattleEndpoint<ID>>,
        active_size: usize,
    ) -> Self {
        let mut active = Vec::with_capacity(active_size);
        let mut count = 0;

        while active.len() < active_size {
            match party.get(count) {
                Some(p) => {
                    if !p.fainted() {
                        active.push(Some(count.into()));
                    }
                }
                None => active.push(None),
            }
            count += 1;
        }

        Self {
            endpoint,
            party: BattleParty {
                id,
                active,
                pokemon: party.into_iter().map(BattlePokemon::from).collect(),
            },
            name,
            settings,
            waiting: false,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unknown")
    }
}

impl<ID: Copy> BattlePlayer<ID> {
    pub fn as_remote(&self) -> RemotePlayer<ID> {
        PlayerKnowable {
            name: self.name.clone(),
            party: PlayerParty {
                id: self.party.id,
                pokemon: self
                    .party
                    .pokemon
                    .iter()
                    .map(|p| p.known.then(|| UnknownPokemon::new(p)))
                    .collect(),
                active: self
                    .party
                    .active
                    .iter()
                    .map(|active| active.as_ref().map(|a| a.index))
                    .collect(),
            },
        }
    }
}
