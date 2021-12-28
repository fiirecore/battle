use core::{cell::Ref, ops::Deref};
use rand::Rng;

use pokedex::{
    item::{bag::SavedBag, Item},
    moves::Move,
    pokemon::{owned::SavedPokemon, party::Party, Pokemon},
    Dex, Initializable, Uninitializable,
};

use crate::{
    data::BattleData,
    endpoint::{BattleEndpoint, ReceiveError},
    message::{ClientMessage, ServerMessage},
    party::{ActivePokemon, PlayerParty},
    player::{ClientPlayerData, Player, PlayerSettings},
};

use super::pokemon::{ActiveBattlePokemon, HostPokemon};

pub type BattlePlayer<ID, P, M, I> =
    Player<ID, ActiveBattlePokemon<ID>, HostPokemon<P, M, I>, I, Box<dyn BattleEndpoint<ID>>>;

pub struct PlayerData<ID> {
    pub id: ID,
    pub name: Option<String>,
    pub party: Party<SavedPokemon>,
    pub bag: SavedBag,
    pub settings: PlayerSettings,
    pub endpoint: Box<dyn BattleEndpoint<ID>>,
}

impl<ID, P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>>
    BattlePlayer<ID, P, M, I>
{
    pub fn send(&mut self, message: ServerMessage<ID>) {
        self.endpoint.send(message)
    }

    pub fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>> {
        self.endpoint.receive()
    }
}

impl<ID> PlayerData<ID> {
    pub(crate) fn init<
        'd,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
    >(
        self,
        random: &mut impl Rng,
        active: usize,
        pokedex: &'d dyn Dex<'d, Pokemon, P>,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
    ) -> BattlePlayer<ID, P, M, I> {
        let pokemon: Party<HostPokemon<P, M, I>> = self
            .party
            .into_iter()
            .flat_map(|p| p.init(random, pokedex, movedex, itemdex))
            .map(Into::into)
            .collect();

        let mut party = PlayerParty::new(self.id, self.name, active, pokemon);

        for index in party.active.iter().flatten().map(ActivePokemon::index) {
            if let Some(pokemon) = party.pokemon.get_mut(index) {
                pokemon.known = true;
            }
        }

        let bag = self.bag.init(itemdex).unwrap_or_default();

        BattlePlayer {
            party,
            bag,
            settings: self.settings,
            endpoint: self.endpoint,
        }
    }
}

impl<ID: Clone> ClientPlayerData<ID> {
    pub fn new<
        'a,
        P: Deref<Target = Pokemon> + 'a + Clone,
        M: Deref<Target = Move> + 'a + Clone,
        I: Deref<Target = Item> + 'a + Clone,
        ITER: Iterator<Item = Ref<'a, BattlePlayer<ID, P, M, I>>>,
    >(
        data: BattleData,
        player: &BattlePlayer<ID, P, M, I>,
        others: ITER,
    ) -> Self
    where
        ID: 'a,
    {
        Self {
            local: PlayerParty {
                id: player.party.id().clone(),
                name: player.party.name.clone(),
                active: ActiveBattlePokemon::as_usize(&player.party.active),
                pokemon: player
                    .party
                    .pokemon
                    .iter()
                    .map(|p| &p.p.p)
                    .cloned()
                    .map(|pokemon| pokemon.uninit())
                    .collect(),
            },
            data,
            remotes: others.map(|player| player.party.as_remote()).collect(),
            bag: player.bag.clone().uninit(),
        }
    }
}
