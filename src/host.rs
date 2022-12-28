//! Basic battle host

use alloc::{collections::BTreeMap, vec::Vec};
use core::hash::Hash;

use rand::Rng;

use pokedex::{item::Item, moves::Move, Dex};

use crate::{
    data::*,
    engine::{BattleEngine, BattlePokemon, ExecuteAction, PlayerQuery},
    message::{ClientMessage, ServerMessage},
    party::{ActivePokemon, PlayerParty},
    player::RemovalReason,
    pokemon::{Indexed, PokemonView, TeamIndex},
    select::{
        BattleSelection, ClientAction, ClientActionType, ClientMoveAction, SelectMessage,
        SelectReason,
    },
};

pub mod moves;
mod party;
mod player;
mod pokemon;
mod state;
// mod timer;
// pub mod saved;

pub use player::{BattlePlayer, PlayerData};

use self::pokemon::ActiveBattlePokemon;

pub(crate) mod prelude {

    pub use super::player::PlayerData;
    // pub use super::Battle;
}

/// A battle host.
pub struct Battle<
    ID: Clone + Ord + Hash + Send + Sync + 'static,
    T: Clone + Send + Sync,
    E: BattleEngine<ID, T>,
> {
    state: state::StateInstance<Option<BattleState>>,
    data: BattleData,
    edata: E::Data,
    players: Vec<BattlePlayer<ID, T>>,
    #[deprecated]
    winner: Option<ID>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BattleState {
    Selecting,
    Moves,
}

// #[deprecated(note = "move")]
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub enum BattleCommand<ID> {
//     /// Team ID + Pokemon Index
//     Faint(TeamIndex<ID>),

//     /// Remove a player from the battle
//     Remove(ID, RemovalReason, firecore_pokedex::Money),
//     /// End game (winner optional)
//     End(Option<ID>),
// }

impl<
        ID: Clone + Ord + Eq + Hash + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
        E: BattleEngine<ID, T>,
    > Battle<ID, T, E>
{
    pub fn new(data: BattleData, players: impl IntoIterator<Item = PlayerData<ID, T>>) -> Self {
        Self {
            state: state::StateInstance::default(),
            data,
            players: players
                .into_iter()
                .map(|player| player.init(data.active))
                .collect(),
            edata: Default::default(),
            winner: Default::default(),
        }
    }

    // pub fn detach(self) {
    //     std::thread::spawn(f)
    // }

    fn begin(&self) {
        for player in self
            .players
            .iter()
            .filter(|player| player.removed.is_none())
        {
            player.send(ServerMessage::PlayerData(
                self.data.clone(),
                PlayerParty {
                    id: player.party.id.clone(),
                    name: player.party.name.clone(),
                    active: ActiveBattlePokemon::as_usize(&player.party.active),
                    pokemon: player
                        .party
                        .pokemon
                        .iter()
                        .map(|p| &p.p)
                        .cloned()
                        .map(|pokemon| pokemon.uninit())
                        .collect(),
                    trainer: player.party.trainer.clone(),
                },
                player.bag.save(),
            ));

            for other in self
                .players
                .iter()
                .filter(|other| other.removed.is_none() && other.party.id != player.party.id)
            {
                player
                    .endpoint
                    .send(ServerMessage::AddOpponent(PlayerParty {
                        id: other.party.id.clone(),
                        name: other.party.name.clone(),
                        active: pokemon::ActiveBattlePokemon::as_usize(&other.party.active),
                        pokemon: other
                            .party
                            .pokemon
                            .iter()
                            .map(BattlePokemon::get_revealed)
                            .collect(),
                        trainer: other.party.trainer.clone(),
                    }));
            }
        }
    }

    fn send_select(&self) {
        for player in self
            .players
            .iter()
            .filter(|player| player.removed.is_none())
        {
            for active in player.party.active.iter().flatten() {
                player.send(ServerMessage::Select(
                    active.index(),
                    SelectMessage::Request(None),
                ));
            }
        }
    }

    fn wait_select(&mut self) {
        if self
            .players
            .iter()
            .filter(|player| player.removed.is_none())
            .all(|player| {
                player
                    .party
                    .active
                    .iter()
                    .flatten()
                    .all(ActiveBattlePokemon::queued)
                    || !player.party.active.iter().any(Option::is_some)
            })
        {
            self.state.set(Some(BattleState::Moves));
        }
    }

    fn queue_moves<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        engine: &E,
        random: &mut R,
        itemdex: &Dex<Item>,
    ) {
        let mut queue = BTreeMap::new();

        for player in self
            .players
            .iter_mut()
            .filter(|player| player.removed.is_none())
        {
            moves::queue_player(
                engine,
                &mut queue,
                &player.party.id,
                &mut player.party.active,
                &mut player.party.pokemon,
                random,
            )
        }

        let queue = queue.into_values().collect();

        let player_queue = self.run_queue(engine, random, itemdex, queue);

        // end queue calculations

        for player in self.players.iter() {
            player.send(ServerMessage::Results(player_queue.clone()));
        }
    }

    fn check_loss(&mut self) {
        let mut i = 0;
        loop {
            if let Some(player) = self.players.get(i) {
                if !player.party.pokemon.iter().any(|p| !p.fainted())
                    || player
                        .party
                        .pokemon
                        .iter()
                        .all(|p| p.moves.iter().all(|m| m.1 == 0))
                        && player.removed.is_none()
                {
                    let id = player.id().clone();
                    self.remove(id, RemovalReason::Loss);
                }
            } else {
                break;
            }
            i += 1;
        }
    }

    fn check_win(&mut self) {
        if self.players.iter().filter(|p| p.removed.is_none()).count() <= 1 {
            self.end(self.players.iter().next().map(|p| p.party.id.clone()))
        }
    }

    fn wait_moves(&mut self) {
        if self
            .players
            .iter()
            .filter(|p| p.removed.is_none())
            .all(|p| !p.party.needs_replace())
        {
            self.state.set(Some(BattleState::Selecting));
        }
    }

    pub fn end(&mut self, winner: Option<ID>) {
        for player in self.players.iter() {
            player.endpoint.send(ServerMessage::End(winner.clone()));
        }
        self.winner = winner.clone();
        self.state.set(None);
    }

    pub fn winner(&self) -> Option<&ID> {
        self.winner.as_ref()
    }

    pub fn running(&self) -> bool {
        self.state.current.0.is_some()
    }

    #[deprecated(note = "needs fixing")]
    pub fn update<R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        engine: &E,
        movedex: &Dex<Move>,
        itemdex: &Dex<Item>,
    ) {
        // self.timer.update(delta);
        self.process(engine, movedex);

        // if self.players.active() <= 1 {
        //     let winner = self.players.keys().next().cloned();
        //     self.end(winner);
        //     return;
        // }

        // self.update_state(engine, random, movedex, itemdex);
    }

    pub fn remove(&mut self, id: ID, reason: RemovalReason) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id() == &id) {
            player.removed = Some(reason);
            for player in self.players.iter() {
                player.send(ServerMessage::Remove(id.clone(), reason.clone(), 0));
            }
        }
    }

    // pub fn faint(&mut self, pokemon: TeamIndex<ID>) {
    //     if let Some(mut team) = self.players.get_mut(pokemon.team()) {
    //         if let Some(pokemon1) = team.party.pokemon.get_mut(pokemon.index()) {
    //             pokemon1.hp = 0;
    //             drop(team);
    //             for mut player in self.players.all_values_mut() {
    //                 player.send(ServerMessage::Command(CommandAction::Faint(
    //                     pokemon.clone(),
    //                 )));
    //             }
    //         }
    //     }
    // }

    fn process(&mut self, engine: &E, movedex: &Dex<Move>) {
        let mut i = 0usize;
        loop {
            if self.players.get(i).is_none() {
                break;
            }
            loop {
                let player = &mut self.players[i];
                match player.endpoint.receive() {
                    Ok(Some(message)) => match message {
                        ClientMessage::Select(active, selection) => {
                            let message = ServerMessage::Select(
                                active,
                                match player
                                    .party
                                    .active
                                    .get(active)
                                    .map(Option::as_ref)
                                    .flatten()
                                    .is_some()
                                {
                                    true => {
                                        match engine.select(
                                            &mut self.edata,
                                            active,
                                            &selection,
                                            player,
                                        ) {
                                            SelectMessage::Request(r) => SelectMessage::Request(r),
                                            confirm => {
                                                player.party.active[active]
                                                    .as_mut()
                                                    .unwrap()
                                                    .queued_move = Some(selection.clone());

                                                confirm
                                            }
                                        }
                                    }
                                    false => {
                                        SelectMessage::Request(Some(SelectReason::MissingPokemon))
                                    }
                                },
                            );

                            player.send(message);
                        }
                        ClientMessage::ReplaceFaint(active, index) => {
                            if match player.party.active_contains(index) {
                                false => match player.party.pokemon.get(index) {
                                    Some(pokemon) => match pokemon.fainted() {
                                        false => {
                                            let id = player.id().clone();
                                            player.party.active[active] = Some(index.into());
                                            let unknown = player.party.reveal_and_get(index);
                                            for other in self
                                                .players
                                                .iter_mut()
                                                .filter(|other| other.party.id != id)
                                            {
                                                if let Some(pokemon) = unknown.as_ref() {
                                                    other.send(ServerMessage::Reveal(Indexed(
                                                        TeamIndex(id.clone(), index),
                                                        PokemonView::Partial(pokemon.clone()),
                                                    )));
                                                }
                                                other.send(ServerMessage::Replace(Indexed(
                                                    TeamIndex(id.clone(), active),
                                                    index,
                                                )));
                                            }
                                            false
                                        }
                                        true => true,
                                    },
                                    None => true,
                                },
                                true => true,
                            } {
                                let player = &self.players[i];
                                player.send(ServerMessage::Replace(Indexed(
                                    TeamIndex(player.party.id.clone(), active),
                                    index,
                                )));
                            }
                        }
                        ClientMessage::TryForfeit => {
                            if player.removed.is_none() {
                                match self.data.versus {
                                    VersusType::Wild => {
                                        let id = player.id().clone();
                                        let reason = RemovalReason::Run;
                                        player.removed = Some(reason);
                                        for player in self.players.iter() {
                                            player.send(ServerMessage::Remove(
                                                id.clone(),
                                                reason.clone(),
                                                0,
                                            ));
                                        }
                                        continue;
                                    }
                                    _ => {
                                        if self.data.settings.allow_forfeit {
                                            player.removed = Some(RemovalReason::Loss);
                                        }
                                    }
                                }
                            }
                        }
                        ClientMessage::LearnMove(pokemon, id, index) => {
                            if let Some(pokemon) = player.party.pokemon.get_mut(pokemon) {
                                if pokemon.learnable.remove(&id) {
                                    if let Some(m) = movedex.try_get(&id) {
                                        pokemon.moves.add(index, m.clone());
                                    }
                                }
                            }
                        }
                    },
                    Ok(None) => break,
                    Err(err) => {
                        todo!();
                    }
                }
            }
            i += 1;
        }
    }

    fn run_queue<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        engine: &E,
        random: &mut R,
        itemdex: &Dex<Item>,
        queue: Vec<Indexed<ID, BattleSelection<ID>>>,
    ) -> Vec<ClientAction<ID>> {
        let mut player_queue = Vec::with_capacity(queue.len());

        let mut index = 0;

        for Indexed(user_id, bmove) in queue {
            match bmove {
                BattleSelection::Move(used_move, target) => {
                    player_queue.push(ClientAction::Announce(
                        index,
                        Some(user_id.clone()),
                        ClientActionType::Move(used_move),
                    ));

                    let actions = match engine.execute(
                        &mut self.edata,
                        random,
                        &mut self.data,
                        ExecuteAction::Move(&used_move, &user_id, target.as_ref()),
                        PlayerQuery(&mut self.players),
                    ) {
                        Ok(actions) => ClientAction::Actions(actions),
                        Err(err) => ClientAction::Error(err.to_string()),
                    };

                    // let mut user = self
                    //     .players
                    //     .iter_mut()
                    //     .find(|p| &p.party.id == user_id.team())
                    //     .unwrap();

                    // if let Some(pokemon) = user.party.active_mut(user_id.index()) {
                    //     if pokemon.fainted() {
                    //         user.party.remove_active(user_id.index());
                    //     }
                    // }

                    player_queue.push(actions);
                }
                BattleSelection::Item(Indexed(target, id)) => {
                    player_queue.push(ClientAction::Announce(
                        index,
                        Some(user_id.clone()),
                        ClientActionType::Item(Indexed(target.clone(), id)),
                    ));

                    let mut actions = Vec::new();

                    let action = match engine.execute(
                        &mut self.edata,
                        random,
                        &mut self.data,
                        ExecuteAction::Item(&id, user_id.team(), target),
                        PlayerQuery(&mut self.players),
                    ) {
                        Ok(results) => ClientAction::Actions(actions),
                        Err(err) => ClientAction::Error(err.to_string()),
                    };

                    player_queue.push(action);
                }
                BattleSelection::Pokemon(new) => {
                    player_queue.push(ClientAction::Announce(
                        index,
                        Some(user_id.clone()),
                        ClientActionType::Pokemon(new),
                    ));

                    match self
                        .players
                        .iter_mut()
                        .find(|p| &p.party.id == user_id.team())
                    {
                        Some(mut user) => {
                            user.party.replace(user_id.index(), Some(new));

                            if let Some(unknown) = user
                                .party
                                .index(user_id.index())
                                .and_then(|index| user.party.reveal_and_get(index))
                            {
                                let id = user.party.id.clone();
                                for mut other in
                                    self.players.iter_mut().filter(|p| p.party.id != id)
                                {
                                    other.send(ServerMessage::Reveal(Indexed(
                                        TeamIndex(id.clone(), new),
                                        PokemonView::Partial(unknown.clone()),
                                    )));
                                }
                            }
                        }
                        None => todo!(),
                    }

                    player_queue.push(ClientAction::Actions(vec![Indexed(
                        user_id,
                        ClientMoveAction::Switch(new),
                    )]));
                }
            }
            index += 1;
        }

        player_queue.push(ClientAction::Announce(index, None, ClientActionType::Post));
        player_queue.push(
            match engine.post(
                &mut self.edata,
                random,
                &mut self.data,
                PlayerQuery(&mut self.players),
            ) {
                Ok(post) => ClientAction::Actions(post),
                Err(err) => ClientAction::Error(err.to_string()),
            },
        );

        player_queue
    }
}
