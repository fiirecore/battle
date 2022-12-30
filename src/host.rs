//! Basic battle host

use alloc::{collections::BTreeMap, vec::Vec};
use core::{fmt::Debug, hash::Hash};

use rand::Rng;

use pokedex::{moves::Move, Dex};

use crate::{
    data::*,
    endpoint::ConnectionError,
    engine::{ActiveBattlePokemon, BattleEngine, BattlePlayer, ExecuteAction, PlayerQuery},
    message::{ClientMessage, ServerMessage},
    party::{ActivePokemon, PlayerParty},
    player::RemovalReason,
    pokemon::{BattlePokemon, Indexed, PokemonView, TeamIndex},
    select::{
        BattleSelection, ClientAction, ClientActionType, PublicAction, SelectConfirm,
        SelectMessage, SelectReason,
    },
};

#[deprecated]
pub static test: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub mod moves;
mod party;
mod player;
// mod timer;
// pub mod saved;

pub use player::PlayerData;

/// A battle host.
pub struct Battle<
    ID: Debug + Clone + Ord + Hash + Send + Sync + 'static,
    T: Clone + Send + Sync,
    E: BattleEngine<ID, T>,
> {
    state: BattleState<ID>,
    data: BattleData,
    edata: E::Data,
    players: PlayerQuery<ID, T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BattleState<ID> {
    Start,
    Selecting(bool),
    Moves(bool),
    Winner(Option<ID>),
}

impl<ID> Default for BattleState<ID> {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug)]
pub struct BattleError<ID>(pub ID, pub BattleErrors);

#[derive(Debug)]
pub enum BattleErrors {
    Connection(ConnectionError),
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
        ID: Debug + Clone + Ord + Eq + Hash + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
        E: BattleEngine<ID, T>,
    > Battle<ID, T, E>
{
    pub fn new(data: BattleData, players: impl IntoIterator<Item = PlayerData<ID, T>>) -> Self {
        Self {
            state: Default::default(),
            data,
            players: PlayerQuery::new(
                players
                    .into_iter()
                    .map(|player| player.init(data.active))
                    .collect(),
            ),
            edata: Default::default(),
        }
    }

    // pub fn detach(self) {
    //     std::thread::spawn(f)
    // }

    pub fn reset(&mut self, engine: &E) {
        self.players.clear();
        engine.reset(&mut self.edata);
        self.state = Default::default();
    }

    pub fn get_data_mut(&mut self) -> &mut BattleData {
        &mut self.data
    }

    pub fn add_players(
        &mut self,
        players: impl IntoIterator<Item = PlayerData<ID, T>>,
    ) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        let mut new_players = Vec::new();
        self.players.extend(players.into_iter().map(|player| {
            new_players.push(player.id.clone());
            player.init(self.data.active)
        }));

        for id in new_players {
            if let Err(errs) = self.send_player_data(
                &self
                    .players
                    .unfiltered_iter()
                    .find(|p| p.id() == &id)
                    .unwrap(),
            ) {
                errors.extend(errs);
            };
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn send_all_player_data(&self) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        for player in self
            .players
            .unfiltered_iter()
            .filter(|player| player.removed.is_none())
        {
            if let Err(errs) = self.send_player_data(player) {
                errors.extend(errs);
            }
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn send_player_data(&self, player: &BattlePlayer<ID, T>) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();

        if let Err(err) = player.send(ServerMessage::PlayerData(
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
        )) {
            errors.push(BattleError(
                player.id().clone(),
                BattleErrors::Connection(err),
            ));
        };

        for other in self
            .players
            .unfiltered_iter()
            .filter(|other| other.removed.is_none() && other.party.id != player.party.id)
        {
            if let Err(err) = player
                .endpoint
                .send(ServerMessage::AddOpponent(PlayerParty {
                    id: other.party.id.clone(),
                    name: other.party.name.clone(),
                    active: ActiveBattlePokemon::as_usize(&other.party.active),
                    pokemon: other
                        .party
                        .pokemon
                        .iter()
                        .map(BattlePokemon::get_revealed)
                        .collect(),
                    trainer: other.party.trainer.clone(),
                }))
            {
                errors.push(BattleError(
                    other.id().clone(),
                    BattleErrors::Connection(err),
                ));
            }
        }

        player.ready();

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn send_select(&self) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        for player in self.players.iter() {
            for index in 0..player.party.active.len() {
                if player
                    .party
                    .active
                    .get(index)
                    .map(Option::as_ref)
                    .flatten()
                    .is_some()
                {
                    if let Err(err) =
                        player.send(ServerMessage::Select(index, SelectMessage::Request(None)))
                    {
                        errors.push(BattleError(
                            player.id().clone(),
                            BattleErrors::Connection(err),
                        ));
                    }
                }
            }
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn wait_select(&mut self) {
        if self.players.iter().all(|player| {
            player
                .party
                .active
                .iter()
                .flatten()
                .all(ActiveBattlePokemon::queued)
                || !player.party.active.iter().any(Option::is_some)
        }) {
            self.state = BattleState::Moves(false);
        }
    }

    fn queue_moves<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        engine: &E,
        random: &mut R,
    ) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        let mut queue = BTreeMap::new();

        for player in self.players.iter_mut() {
            moves::queue_player(
                engine,
                &mut queue,
                &player.party.id,
                &mut player.party.active,
                &mut player.party.pokemon,
                random,
            )
        }

        let queue = queue.into_values().collect::<Vec<_>>();

        let player_queue = self.run_queue(engine, random, queue);

        // end queue calculations

        for player in self.players.unfiltered_iter().filter(|p| p.is_ready()) {
            if let Err(err) = player.send(ServerMessage::Results(player_queue.clone())) {
                errors.push(BattleError(
                    player.id().clone(),
                    BattleErrors::Connection(err),
                ));
            }
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn check_loss(&mut self) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        let mut i = 0;
        loop {
            if let Some(player) = self.players.get_index(i) {
                if player.removed.is_none()
                    && (!player.party.pokemon.iter().any(|p| !p.fainted())
                        || player
                            .party
                            .pokemon
                            .iter()
                            .all(|p| p.moves.iter().all(|m| m.is_empty())))
                {
                    println!("loss");
                    let id = player.id().clone();
                    if let Err(errs) = self.remove(id, RemovalReason::Loss) {
                        errors.extend(errs);
                    }
                }
            } else {
                break;
            }
            i += 1;
        }
        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn check_win(&mut self) -> Result<(), Vec<BattleError<ID>>> {
        if self.players.iter().count() <= 1 {
            self.end(self.players.iter().next().map(|p| p.party.id.clone()))?;
        }
        Ok(())
    }

    fn wait_moves(&mut self) {
        if self
            .players
            .iter()
            .filter(|p| p.removed.is_none())
            .all(|p| !p.party.needs_replace())
        {
            self.state = BattleState::Selecting(false);
        }
        #[cfg(debug_assertions)]
        {
            if let Some(p) = self
                .players
                .iter()
                .filter(|p| p.removed.is_none())
                .find(|p| p.party.needs_replace())
            {
                println!(
                    "AI #{:?}, {:?}",
                    p.id(),
                    p.party
                        .pokemon
                        .iter()
                        .map(|p| format!("HP {}/{}", p.hp, p.max_hp()))
                        .collect::<Vec<_>>()
                );
                test.store(
                    unsafe { *(p.id() as *const ID as *const u8) },
                    std::sync::atomic::Ordering::Relaxed,
                );
            }
        }
    }

    pub fn end(&mut self, winner: Option<ID>) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        for player in self.players.unfiltered_iter() {
            if let Err(err) = player.endpoint.send(ServerMessage::End(winner.clone())) {
                errors.push(BattleError(
                    player.id().clone(),
                    BattleErrors::Connection(err),
                ));
            }
        }
        self.state = BattleState::Winner(winner);
        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    pub fn winner(&self) -> Option<Option<&ID>> {
        match &self.state {
            BattleState::Winner(winner) => Some(winner.as_ref()),
            _ => None,
        }
    }

    pub fn running(&self) -> bool {
        !matches!(self.state, BattleState::Winner(..))
    }

    pub fn update<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        random: &mut R,
        engine: &E,
        movedex: &Dex<Move>,
    ) -> Result<(), Vec<BattleError<ID>>> {
        println!(
            "state: {:?}, players left: {}",
            self.state,
            self.players.iter().count()
        );

        let mut errors = Vec::new();
        // self.timer.update(delta);
        if let Err(errs) = self.process(engine, movedex) {
            errors.extend(errs);
        }

        match &mut self.state {
            BattleState::Start => {
                if let Err(errs) = self.send_all_player_data() {
                    errors.extend(errs);
                }
                self.state = BattleState::Selecting(false);
            }
            BattleState::Selecting(wait) => {
                match wait {
                    false => {
                        *wait = true;
                        if let Err(errs) = self.send_select() {
                            errors.extend(errs);
                        }
                    }
                    true => self.wait_select(),
                }

                if let Err(errs) = self.check_win() {
                    errors.extend(errs);
                }
            }
            BattleState::Moves(wait) => {
                match wait {
                    false => {
                        *wait = true;
                        if let Err(errs) = self.queue_moves(engine, random) {
                            errors.extend(errs);
                        }
                        if let Err(errs) = self.check_loss() {
                            errors.extend(errs);
                        }
                    }
                    true => self.wait_moves(),
                }

                if let Err(errs) = self.check_win() {
                    errors.extend(errs);
                }
            }
            BattleState::Winner(..) => (),
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    pub fn remove(&mut self, id: ID, reason: RemovalReason) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        if let Some(player) = self.players.get_mut(&id) {
            player.removed = Some(reason);
            for player in self.players.unfiltered_iter() {
                if let Err(err) = player.send(ServerMessage::Remove(id.clone(), reason.clone(), 0))
                {
                    errors.push(BattleError(
                        player.id().clone(),
                        BattleErrors::Connection(err),
                    ));
                }
            }
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
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

    fn process(&mut self, engine: &E, movedex: &Dex<Move>) -> Result<(), Vec<BattleError<ID>>> {
        let mut errors = Vec::new();
        let mut i = 0usize;
        loop {
            if self.players.get_index(i).is_none() {
                break;
            }
            loop {
                let player = self.players.get_index_mut(i).unwrap();
                match player.endpoint.receive() {
                    Ok(Some(message)) => match message {
                        ClientMessage::Select(active, selection) => {
                            let select = match player.party.active.len() > active {
                                true => {
                                    match selection {
                                        BattleSelection::Pokemon(new) => {
                                            match player.party.pokemon.get(active) {
                                                Some(pokemon) => match pokemon.fainted() {
                                                    true => {
                                                        if !player.party.active_contains(new) {
                                                            let id = player.id().clone();
                                                            player.party.active[active] =
                                                                Some(new.into());
                                                            let unknown =
                                                                player.party.reveal_and_get(new);
    
                                                            for player in self
                                                                .players
                                                                .unfiltered_iter_mut()
                                                                .filter(|p| p.is_ready())
                                                            {
                                                                if &id != player.id() {
                                                                    if let Some(pokemon) =
                                                                        unknown.as_ref()
                                                                    {
                                                                        if let Err(err) = player.send(
                                                                        ServerMessage::Reveal(Indexed(
                                                                            TeamIndex(id.clone(), new),
                                                                            PokemonView::Partial(
                                                                                pokemon.clone(),
                                                                            ),
                                                                        )),
                                                                    ) {
                                                                        errors.push(BattleError(
                                                                            player.id().clone(),
                                                                            BattleErrors::Connection(err),
                                                                        ));
                                                                    }
                                                                    }
                                                                }
                                                                if let Err(err) = player.send(
                                                                    ServerMessage::Replace(Indexed(
                                                                        TeamIndex(id.clone(), active),
                                                                        new,
                                                                    )),
                                                                ) {
                                                                    errors.push(BattleError(
                                                                        player.id().clone(),
                                                                        BattleErrors::Connection(err),
                                                                    ));
                                                                }
                                                            }
                                                            SelectMessage::Confirm(SelectConfirm::Other)
                                                        } else {
                                                            SelectMessage::Request(Some(SelectReason::InvalidInput))
                                                        }
                                                    }
                                                    false => {
                                                        if player.party.active_contains(new)
                                                            || new >= player.party.pokemon.len()
                                                        {
                                                            SelectMessage::Request(Some(
                                                                SelectReason::MissingAction,
                                                            ))
                                                        } else {
                                                            SelectMessage::Confirm(SelectConfirm::Other)
                                                        }
                                                    }
                                                },
                                                None => SelectMessage::Request(Some(SelectReason::MissingPokemon)),
                                            }
                                        }
                                        selection => match player.party.active[active].is_some() {
                                            true => match engine.select(
                                                &mut self.edata,
                                                active,
                                                &selection,
                                                player,
                                            ) {
                                                SelectMessage::Request(r) => SelectMessage::Request(r),
                                                confirm => {
                                                    player.party.active[active].as_mut().unwrap()
                                                        .queued_move = Some(selection.clone());
    
                                                    confirm
                                                }
                                            },
                                            false => SelectMessage::Request(Some(SelectReason::MissingActive)),
                                        },
                                    }
                                }
                                false => SelectMessage::Request(Some(SelectReason::InvalidInput)),
                            };

                            let player = self.players.get_index_mut(i).unwrap();

                            if let Err(err) = player.send(ServerMessage::Select(active, select)) {
                                errors.push(BattleError(
                                    player.id().clone(),
                                    BattleErrors::Connection(err),
                                ));
                            }
                        }
                        ClientMessage::TryForfeit => {
                            if player.removed.is_none() {
                                match self.data.versus {
                                    VersusType::Wild => {
                                        player.removed = Some(RemovalReason::Run);
                                    }
                                    _ => {
                                        if self.data.settings.allow_forfeit {
                                            player.removed = Some(RemovalReason::Loss);
                                        }
                                    }
                                }
                                if let Some(reason) = player.removed.clone() {
                                    let id = player.id().clone();
                                    for player in self.players.unfiltered_iter() {
                                        if let Err(err) = player.send(ServerMessage::Remove(
                                            id.clone(),
                                            reason.clone(),
                                            0,
                                        )) {
                                            errors.push(BattleError(
                                                player.id().clone(),
                                                BattleErrors::Connection(err),
                                            ));
                                        }
                                    }
                                    continue;
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
                        errors.push(BattleError(
                            player.id().clone(),
                            BattleErrors::Connection(err),
                        ));
                    }
                }
            }
            i += 1;
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn run_queue<R: Rng + Clone + Send + Sync + 'static>(
        &mut self,
        engine: &E,
        random: &mut R,
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

                    if self
                        .players
                        .get(user_id.team())
                        .and_then(|p| p.party.active(user_id.index()))
                        .is_some()
                    {
                        let actions = match engine.execute(
                            &mut self.edata,
                            random,
                            &mut self.data,
                            ExecuteAction::Move(&used_move, &user_id, target.as_ref()),
                            &mut self.players,
                        ) {
                            Ok(actions) => ClientAction::Actions(actions),
                            Err(err) => ClientAction::Error(err.to_string()),
                        };

                        player_queue.push(actions);
                    }

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
                }
                BattleSelection::Item(Indexed(target, id)) => {
                    player_queue.push(ClientAction::Announce(
                        index,
                        Some(user_id.clone()),
                        ClientActionType::Item(Indexed(target.clone(), id)),
                    ));

                    let action = match engine.execute(
                        &mut self.edata,
                        random,
                        &mut self.data,
                        ExecuteAction::Item(&id, user_id.team(), target),
                        &mut self.players,
                    ) {
                        Ok(results) => ClientAction::Actions(results),
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
                        Some(user) => {
                            user.party.replace(user_id.index(), Some(new));

                            if let Some(unknown) = user
                                .party
                                .index(user_id.index())
                                .and_then(|index| user.party.reveal_and_get(index))
                            {
                                let id = user.party.id.clone();
                                for other in self.players.iter_mut() {
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
                        PublicAction::Switch(new),
                    )]));
                }
            }
            index += 1;
        }

        player_queue.push(ClientAction::Announce(index, None, ClientActionType::Post));
        player_queue.push(
            match engine.post(&mut self.edata, random, &mut self.data, &mut self.players) {
                Ok(post) => ClientAction::Actions(post),
                Err(err) => ClientAction::Error(err.to_string()),
            },
        );

        player_queue
    }
}
