use pokedex::pokemon::Party;

use crate::{
    party::{BattleParty, PlayerParty},
    player::UninitRemotePlayer,
    pokemon::{battle::BattlePokemon, UnknownPokemon, OwnedRefPokemon},
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

pub struct BattlePlayer<'d, ID> {
    pub endpoint: Box<dyn BattleEndpoint<ID>>,
    pub party: BattleParty<'d, ID>,
    pub name: Option<String>,
    pub settings: PlayerSettings,
    /// Player's turn has finished
    pub waiting: bool,
}

impl<'d, ID> BattlePlayer<'d, ID> {
    pub fn new(
        id: ID,
        party: Party<OwnedRefPokemon<'d>>,
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

impl<'d, ID: Copy> BattlePlayer<'d, ID> {
    pub fn as_remote(&self) -> UninitRemotePlayer<ID> {
        PlayerKnowable {
            name: self.name.clone(),
            party: PlayerParty {
                id: self.party.id,
                pokemon: self
                    .party
                    .pokemon
                    .iter()
                    .map(|p| p.known.then(|| UnknownPokemon::new(p).uninit()))
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
