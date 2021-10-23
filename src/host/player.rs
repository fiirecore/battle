use core::cell::Ref;
use rand::Rng;

use pokedex::{item::Item, moves::Move, pokemon::Pokemon, Dex};

use crate::{
    player::{PlayerWithEndpoint, Player, ValidatedPlayer},
    BattleData,
};

use super::pokemon::{ActivePokemon, BattlePokemon};

pub type BattlePlayer<'d, ID, const AS: usize> =
    Player<ID, ActivePokemon<ID>, BattlePokemon<'d>, AS>;

impl<ID, const AS: usize> PlayerWithEndpoint<ID, AS> {
    pub(crate) fn init<'d>(
        self,
        random: &mut impl Rng,
        pokedex: &'d impl Dex<Pokemon>,
        movedex: &'d impl Dex<Move>,
        itemdex: &'d impl Dex<Item>,
    ) -> BattlePlayer<'d, ID, AS> {
        let pokemon = self.0
            .party
            .into_iter()
            .flat_map(|p| p.init(random, pokedex, movedex, itemdex))
            .map(Into::into)
            .collect();
        BattlePlayer::new(self.0.id, self.0.name, pokemon, self.0.settings, self.1)
    }
}

impl<ID: Clone, const AS: usize> ValidatedPlayer<ID, AS> {
    pub fn new<'d: 'a, 'a, I: Iterator<Item = Ref<'a, BattlePlayer<'d, ID, AS>>> + 'a>(
        data: BattleData,
        player: &BattlePlayer<ID, AS>,
        others: I,
    ) -> Self
    where
        ID: 'a,
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
