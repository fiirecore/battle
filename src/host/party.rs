use crate::{
    party::{PlayerParty, RemoteParty},
    pokemon::{remote::RemotePokemon, BattlePokemon}, engine::ActiveBattlePokemon,
};

pub type BattleParty<ID, T> = PlayerParty<ID, ActiveBattlePokemon<ID>, BattlePokemon, T>;

impl<ID, T> BattleParty<ID, T> {
    pub fn reveal_and_get(&mut self, index: usize) -> Option<RemotePokemon> {
        self.pokemon.get_mut(index).and_then(|p| {
            p.reveal();
            p.get_revealed()
        })
    }

    pub fn ready_to_move(&self) -> bool {
        self.active
            .iter()
            .flatten()
            .all(ActiveBattlePokemon::queued)
            || self.active.iter().all(Option::is_none)
    }

    pub fn as_remote(&self) -> RemoteParty<ID, T>
    where
        ID: Clone,
        T: Clone,
    {
        RemoteParty {
            id: self.id.clone(),
            name: self.name.clone(),
            pokemon: self
                .pokemon
                .iter()
                .map(BattlePokemon::get_revealed)
                .collect(),
            active: ActiveBattlePokemon::as_usize(&self.active),
            trainer: self.trainer.clone(),
        }
    }
}
