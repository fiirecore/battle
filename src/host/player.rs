use core::cell::Ref;
use rand::Rng;

use pokedex::{
    item::Item,
    moves::Move,
    pokemon::{Pokemon},
    Dex,
};

use crate::{
    player::{Player, LocalPlayer, ValidatedPlayer},
    pokemon::ActivePokemon,
    BattleEndpoint, BattleData,
};

use super::pokemon::BattlePokemon;

pub type BattlePlayer<'d, ID, E, const AS: usize> =
    Player<ID, ActivePokemon<ID>, BattlePokemon<'d>, E, AS>;

impl<ID, E: BattleEndpoint<ID, AS>, const AS: usize> LocalPlayer<ID, E, AS> {
    pub(crate) fn init<'d>(
        self,
        random: &mut impl Rng,
        pokedex: &'d impl Dex<Pokemon>,
        movedex: &'d impl Dex<Move>,
        itemdex: &'d impl Dex<Item>,
    ) -> BattlePlayer<'d, ID, E, AS> {
        let pokemon = self
            .party
            .into_iter()
            .flat_map(|p| p.init(random, pokedex, movedex, itemdex))
            .map(Into::into)
            .collect();
        BattlePlayer::new(self.id, self.name, pokemon, self.settings, self.endpoint)
    }
}

impl<ID: Clone, const AS: usize> ValidatedPlayer<ID, AS> {
    pub fn new<
        'd: 'a,
        'a,
        E: BattleEndpoint<ID, AS>,
        I: Iterator<Item = Ref<'a, BattlePlayer<'d, ID, E, AS>>> + 'a,
    >(
        data: BattleData,
        player: &BattlePlayer<ID, E, AS>,
        others: I,
    ) -> Self
    where
        ID: 'a,
        E: 'a,
    {
        Self {
            id: player.party.id().clone(),
            name: player.party.name.clone(),
            active: ActivePokemon::into_remote(&player.party.active),
            data,
            remotes: others.map(|player| player.party.as_remote()).collect(),
        }
    }
}