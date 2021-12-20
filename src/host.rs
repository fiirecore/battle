//! Basic battle host

use core::{cell::RefMut, cmp::Reverse, fmt::Display, hash::Hash, ops::Deref};
use log::warn;
use rand::Rng;

use pokedex::{
    item::Item,
    moves::{Move, Priority},
    pokemon::{
        stat::{BaseStat, StatType},
        Health, Pokemon,
    },
    Dex,
};

use crate::{
    data::*,
    endpoint::ReceiveError,
    item::engine::{ItemEngine, ItemResult},
    message::{
        ClientMessage, EndMessage, FailedAction, ServerMessage, StartableAction, TimedAction,
    },
    moves::{
        damage::{ClientDamage, DamageResult},
        engine::{MoveEngine, MoveResult},
        BattleMove, ClientMove, ClientMoveAction,
    },
    player::ClientPlayerData,
    pokemon::{Indexed, PokemonIdentifier},
};

mod collections;
mod party;
mod player;
mod pokemon;
mod timer;

use collections::BattleMap;
use party::BattleParty;
use player::{BattlePlayer, PlayerData};
use pokemon::HostPokemon;
use std::collections::BTreeMap;
use timer::*;

pub(crate) mod prelude {

    pub use super::player::PlayerData;
    pub use super::Battle;
}

/// A battle host.
pub struct Battle<
    ID: core::fmt::Debug + Display + Clone + Ord + Hash + 'static,
    P: Deref<Target = Pokemon> + Clone,
    M: Deref<Target = Move> + Clone,
    I: Deref<Target = Item> + Clone,
> {
    state: BattleState<ID>,
    data: BattleData,
    players: BattleMap<ID, BattlePlayer<ID, P, M, I>>, // can change to dex implementation, and impl Identifiable for BattlePlayer
    timer: Timer,
}

#[derive(Debug)]
enum BattleState<ID> {
    Start,
    StartSelecting,
    WaitSelecting,
    QueueMoves,
    WaitReplace,
    End(Option<ID>),
}

impl<ID> Default for BattleState<ID> {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovePriority<ID: Ord> {
    First(ID, usize),
    Second(Reverse<Priority>, Reverse<BaseStat>, Option<u16>),
}

impl<
        ID: core::fmt::Debug + Display + Clone + Ord + Hash + 'static,
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
    > Battle<ID, P, M, I>
{
    pub fn new<'d, R: Rng>(
        data: BattleData,
        random: &mut R,
        active: usize,
        pokedex: &'d dyn Dex<'d, Pokemon, P>,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
        players: impl Iterator<Item = PlayerData<ID>>,
    ) -> Self {
        Self {
            players: players
                .map(|p| {
                    let p = p.init(random, active, pokedex, movedex, itemdex);
                    (p.id().clone(), p)
                })
                .collect(),
            state: BattleState::default(),
            data,
            timer: Default::default(),
        }
    }

    pub fn begin(&mut self) {
        for mut player in self.players.values_mut() {
            let v = ClientPlayerData::new(self.data, &player, self.players.values());
            player.send(ServerMessage::Begin(v));
        }

        self.state = BattleState::StartSelecting;
    }

    pub fn end(&mut self, winner: Option<ID>) {
        for (id, mut player) in self.players.iter_mut() {
            match Some(id) == winner.as_ref() {
                true => player.send(ServerMessage::PlayerEnd(EndMessage::Win)),
                false => player.send(ServerMessage::PlayerEnd(EndMessage::Lose)),
            }
            player.send(ServerMessage::GameEnd(winner.clone()))
        }
        self.state = BattleState::End(winner);
    }

    pub fn update<'d, R: Rng + Clone + 'static, ENG: MoveEngine + ItemEngine>(
        &mut self,
        random: &mut R,
        engine: &mut ENG,
        delta: f32,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
    ) {
        self.timer.update(delta);

        for player in self.players.values_mut() {
            self.receive(player, movedex);
        }

        self.update_state(random, engine, movedex, itemdex);
    }

    fn update_state<'d, R: Rng + Clone + 'static, ENG: MoveEngine + ItemEngine>(
        &mut self,
        random: &mut R,
        engine: &mut ENG,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
    ) {
        match self.state {
            BattleState::Start => self.begin(),
            BattleState::StartSelecting => {
                for mut player in self.players.values_mut() {
                    player.send(ServerMessage::Start(StartableAction::Selecting));
                }
                self.state = BattleState::WaitSelecting;
            }
            BattleState::WaitSelecting => {
                if self.players.values().all(|p| p.party.ready_to_move()) {
                    self.state = BattleState::QueueMoves;
                } else if self.timer.wait(TimedAction::Selecting) {
                    for mut player in self
                        .players
                        .values_mut()
                        .filter(|p| !p.party.ready_to_move())
                    {
                        player.send(ServerMessage::Ping(TimedAction::Selecting));
                    }
                }
            }
            BattleState::QueueMoves => {
                let queue = move_queue(&mut self.players, random);

                let player_queue = self.run_queue(random, engine, movedex, itemdex, queue);

                // end queue calculations

                for mut player in self.players.values_mut() {
                    player.send(ServerMessage::Start(StartableAction::Turns(
                        player_queue.clone(),
                    )));
                }

                for (id, player) in self.players.iter() {
                    if player.party.all_fainted() {
                        self.remove_player(id, EndMessage::Lose);
                    }
                }

                if self.players.active() <= 1 {
                    let winner = self.players.keys().next().cloned();
                    self.end(winner);
                    return;
                }

                self.state = BattleState::WaitReplace;
                self.update_state(random, engine, movedex, itemdex);
            }
            BattleState::WaitReplace => {
                match self.players.values().all(|p| !p.party.needs_replace()) {
                    true => self.state = BattleState::StartSelecting,
                    false => {
                        if self.timer.wait(TimedAction::Replace) {
                            for mut player in self
                                .players
                                .values_mut()
                                .filter(|p| p.party.needs_replace())
                            {
                                player.send(ServerMessage::Ping(TimedAction::Replace));
                            }
                        }
                    }
                }
            }
            BattleState::End(..) => (),
        }
    }

    pub fn remove_player(&self, id: &ID, reason: EndMessage) {
        if let Some(mut player) = self.players.deactivate(id) {
            player.send(ServerMessage::PlayerEnd(reason));
        }
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, BattleState::End(..))
    }

    pub fn data(&self) -> &BattleData {
        &self.data
    }

    pub fn winner(&self) -> Option<&ID> {
        match &self.state {
            BattleState::End(winner) => winner.as_ref(),
            _ => None,
        }
    }

    fn receive<'d>(
        &self,
        mut player: RefMut<BattlePlayer<ID, P, M, I>>,
        movedex: &'d dyn Dex<'d, Move, M>,
    ) {
        loop {
            match player.receive() {
                Ok(message) => match message {
                    ClientMessage::Move(active, bmove) => {
                        if match &bmove {
                            BattleMove::Move(index, ..) => {
                                // maybe add check for incorrect targeting here?
                                if !player
                                    .party
                                    .active(active)
                                    .map(|p| p.moves.len() > *index)
                                    .unwrap_or_default()
                                {
                                    player.send(ServerMessage::Fail(FailedAction::Move(active)));
                                    false
                                } else {
                                    true
                                }
                            }
                            BattleMove::Switch(to) => {
                                if player.party.active_contains(*to) {
                                    player.send(ServerMessage::Fail(FailedAction::Switch(active)));
                                    false
                                } else {
                                    true
                                }
                            }
                            BattleMove::UseItem(..) => true,
                        } {
                            match player
                                .party
                                .active
                                .get_mut(active)
                                .map(Option::as_mut)
                                .flatten()
                            {
                                Some(pokemon) => pokemon.queued_move = Some(bmove),
                                None => warn!(
                                    "Party {} could not add move #{} to pokemon #{}",
                                    player.party.name(),
                                    bmove,
                                    active
                                ),
                            }
                        }
                    }
                    ClientMessage::ReplaceFaint(active, index) => {
                        if match player.party.active_contains(index) {
                            false => match player.party.pokemon.get(index) {
                                Some(pokemon) => match pokemon.fainted() {
                                    false => {
                                        player.party.active[active] = Some(index.into());
                                        let unknown = player.party.know(index);
                                        for mut other in self.players.values_mut() {
                                            let id =
                                                PokemonIdentifier(player.party.id().clone(), index);
                                            if let Some(pokemon) = unknown.as_ref() {
                                                other.send(ServerMessage::AddRemote(Indexed(
                                                    id.clone(),
                                                    pokemon.clone(),
                                                )));
                                            }
                                            other.send(ServerMessage::Replace(Indexed(
                                                PokemonIdentifier(id.team().clone(), index),
                                                id.index(),
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
                            player.send(ServerMessage::Fail(FailedAction::Replace(active)));
                        }
                    }
                    ClientMessage::Forfeit => {
                        self.remove_player(player.id(), EndMessage::Lose);
                    }
                    ClientMessage::LearnMove(pokemon, id, index) => {
                        if let Some(pokemon) = player.party.pokemon.get_mut(pokemon) {
                            if pokemon.learnable_moves.remove(&id) {
                                if let Some(m) = movedex.try_get(&id) {
                                    pokemon.moves.add(index, m);
                                }
                            }
                        }
                    }
                },
                Err(err) => match err {
                    Some(err) => match err {
                        ReceiveError::Disconnected => {
                            self.remove_player(player.id(), EndMessage::Lose)
                        }
                    },
                    None => break,
                },
            }
        }
    }

    fn run_queue<'d, R: Rng + Clone + 'static, ENG: MoveEngine + ItemEngine>(
        &mut self,
        random: &mut R,
        engine: &mut ENG,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
        queue: Vec<Indexed<ID, BattleMove<ID>>>,
    ) -> Vec<Indexed<ID, ClientMove<ID>>> {
        let mut player_queue = Vec::with_capacity(queue.len());

        for Indexed(user_id, bmove) in queue {
            match bmove {
                BattleMove::Move(move_index, targets) => {
                    let turn = match self.players.get(user_id.team()) {
                        Some(user) => match user.party.active(user_id.index()) {
                            Some(pokemon) => {
                                // match Some(pokemon.moves.get(0).unwrap().0)
                                match pokemon.moves.get(move_index) {
                                    Some(m) => {
                                        if m.pp() != 0 {
                                            let used_move = &*m.0;
                                            match MoveEngine::execute(
                                                engine,
                                                random,
                                                used_move,
                                                Indexed(user_id.clone(), pokemon),
                                                targets,
                                                &self.players,
                                            ) {
                                                Ok(turn) => Some((used_move.id, turn)),
                                                Err(err) => {
                                                    warn!("Cannot execute move {} for user {}'s pokemon {} with error {}", used_move.name, user.name(), pokemon.name(), err);
                                                    None
                                                }
                                            }
                                        } else {
                                            log::debug!("{}'s {} does not have enough PP to use their move!", user.name(), pokemon.name());
                                            None
                                        }
                                    }
                                    None => {
                                        warn!(
                                            "Could not get move #{} for {}'s {}",
                                            move_index,
                                            user.name(),
                                            pokemon.name()
                                        );
                                        None
                                    }
                                }
                            }
                            None => None,
                        },
                        None => {
                            warn!("Cannot get user {}!", user_id.team());
                            None
                        }
                    };

                    if let Some((used_move, output)) = turn {
                        let mut actions = Vec::with_capacity(output.len());

                        for Indexed(target_id, action) in output {
                            match self.players.get_mut(target_id.team()) {
                                Some(mut player) => {
                                    match player.party.active_mut(target_id.index()) {
                                        Some(target) => {
                                            /// calculates hp and adds it to actions
                                            fn on_damage<
                                                P: Deref<Target = Pokemon>,
                                                M: Deref<Target = Move>,
                                                I: Deref<Target = Item>,
                                                ID,
                                            >(
                                                location: PokemonIdentifier<ID>,
                                                pokemon: &mut HostPokemon<P, M, I>,
                                                actions: &mut Vec<Indexed<ID, ClientMoveAction>>,
                                                result: DamageResult<Health>,
                                            ) {
                                                pokemon.hp =
                                                    pokemon.hp.saturating_sub(result.damage);
                                                actions.push(Indexed(
                                                    location,
                                                    ClientMoveAction::SetHP(ClientDamage::Result(
                                                        DamageResult {
                                                            damage: pokemon.percent_hp(),
                                                            effective: result.effective,
                                                            crit: result.crit,
                                                        },
                                                    )),
                                                ));
                                            }

                                            let t_id = target_id.clone();

                                            match action {
                                                MoveResult::Damage(result) => on_damage(
                                                    target_id,
                                                    target,
                                                    &mut actions,
                                                    result,
                                                ),
                                                MoveResult::Ailment(ailment) => {
                                                    target.ailment = Some(ailment);
                                                    actions.push(Indexed(
                                                        target_id,
                                                        ClientMoveAction::Ailment(ailment),
                                                    ));
                                                }
                                                MoveResult::Heal(health) => {
                                                    let hp = health.abs() as u16;
                                                    target.hp = match health.is_positive() {
                                                        true => target.hp + hp.min(target.max_hp()),
                                                        false => target.hp.saturating_sub(hp),
                                                    };
                                                    actions.push(Indexed(
                                                        target_id,
                                                        ClientMoveAction::SetHP(
                                                            ClientDamage::Number(
                                                                target.percent_hp(),
                                                            ),
                                                        ),
                                                    ));
                                                }
                                                MoveResult::Stat(stat, stage) => {
                                                    target.stages.change_stage(stat, stage);
                                                    actions.push(Indexed(
                                                        target_id,
                                                        ClientMoveAction::AddStat(stat, stage),
                                                    ));
                                                }
                                                MoveResult::Flinch => actions.push(Indexed(
                                                    target_id,
                                                    ClientMoveAction::Flinch,
                                                )),
                                                MoveResult::Miss => {
                                                    actions.push(Indexed(
                                                        target_id,
                                                        ClientMoveAction::Miss,
                                                    ));
                                                    continue;
                                                }
                                                MoveResult::Error => actions.push(Indexed(
                                                    target_id,
                                                    ClientMoveAction::Error,
                                                )),
                                            }

                                            if target.fainted() {
                                                let experience =
                                                    target.battle_exp_from(&self.data.type_);

                                                player.party.remove_active(t_id.index());

                                                if player.id() != t_id.team() {
                                                    let mut user = self
                                                        .players
                                                        .get_mut(user_id.team())
                                                        .unwrap();

                                                    if user.settings.gains_exp {
                                                        let user = user
                                                            .party
                                                            .active_mut(user_id.index())
                                                            .unwrap();

                                                        user.learnable_moves.extend(
                                                            user.p.add_exp(movedex, experience),
                                                        );

                                                        actions.push(Indexed(
                                                            user_id.clone(),
                                                            ClientMoveAction::SetExp(
                                                                experience, user.level,
                                                            ),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                None => {}
                            }
                        }

                        let mut user = self.players.get_mut(user_id.team()).unwrap();

                        if let Some(pokemon) = user.party.active_mut(user_id.index()) {
                            // decrement PP
                            let pp = &mut pokemon.moves[move_index].1;
                            *pp = pp.saturating_sub(1);

                            if pokemon.fainted() {
                                user.party.remove_active(user_id.index());
                            }
                        }

                        const TEMP_PP: u8 = 1;

                        player_queue.push(Indexed(
                            user_id,
                            ClientMove::Move(used_move, TEMP_PP, actions),
                        ));
                    }
                }
                BattleMove::UseItem(Indexed(target, id)) => match itemdex.try_get(&id) {
                    Some(item) => {
                        match ItemEngine::execute(
                            engine,
                            &self.data,
                            random,
                            &*item,
                            user_id.team(),
                            target.clone(),
                            &mut self.players,
                        ) {
                            Ok(results) => {
                                player_queue.push(Indexed(
                                    user_id.clone(),
                                    ClientMove::UseItem(Indexed(target, id)),
                                ));

                                let mut user = match self.players.get_mut(user_id.team()) {
                                    Some(user) => user,
                                    None => continue,
                                };

                                for result in results {
                                    match result {
                                        ItemResult::Catch(pokemon) => {
                                            user.send(ServerMessage::Catch(pokemon))
                                        }
                                    }
                                }
                            }
                            Err(err) => warn!("Cannot execute item engine with error {}", err),
                        }
                    }
                    None => warn!("Could not get item with id {}", id),
                },
                BattleMove::Switch(new) => match self.players.get_mut(user_id.team()) {
                    Some(mut user) => {
                        user.party.replace(user_id.index(), Some(new));

                        if let Some(unknown) = user
                            .party
                            .index(user_id.index())
                            .map(|index| user.party.know(index))
                            .flatten()
                        {
                            for mut other in self.players.values_mut() {
                                other.send(ServerMessage::AddRemote(Indexed(
                                    PokemonIdentifier(user.party.id().clone(), new),
                                    unknown.clone(),
                                )))
                            }
                        }
                        player_queue.push(Indexed(user_id, ClientMove::Switch(new)));
                    }
                    None => todo!(),
                },
            }
        }
        player_queue
    }
}

pub fn move_queue<
    ID: Clone + Ord + Hash,
    R: Rng,
    P: Deref<Target = Pokemon>,
    M: Deref<Target = Move>,
    I: Deref<Target = Item>,
>(
    players: &mut BattleMap<ID, BattlePlayer<ID, P, M, I>>,
    random: &mut R,
) -> Vec<Indexed<ID, BattleMove<ID>>> {
    let mut queue = BTreeMap::new();

    for mut player in players.values_mut() {
        queue_player(&mut queue, &mut player.party, random)
    }

    queue.into_values().collect()
}

fn queue_player<
    ID: Clone + Ord,
    R: Rng,
    P: Deref<Target = Pokemon>,
    M: Deref<Target = Move>,
    I: Deref<Target = Item>,
>(
    queue: &mut BTreeMap<MovePriority<ID>, Indexed<ID, BattleMove<ID>>>,
    party: &mut BattleParty<ID, P, M, I>,
    random: &mut R,
) {
    for index in 0..party.active.len() {
        if let Some(pokemon) = party.active.get_mut(index).map(Option::as_mut).flatten() {
            if let Some(action) = pokemon.queued_move.take() {
                if let Some(instance) = party.active(index) {
                    let pokemon = PokemonIdentifier(party.id().clone(), index);

                    let mut priority = match action {
                        BattleMove::Move(index, ..) => MovePriority::Second(
                            Reverse(
                                instance
                                    .moves
                                    .get(index)
                                    .map(|i| i.0.priority)
                                    .unwrap_or_default(),
                            ),
                            Reverse(instance.stat(StatType::Speed)),
                            None,
                        ),
                        _ => MovePriority::First(party.id().clone(), index),
                    };

                    fn tie_break<ID: Ord, R: Rng>(
                        queue: &mut BTreeMap<MovePriority<ID>, Indexed<ID, BattleMove<ID>>>,
                        random: &mut R,
                        priority: &mut MovePriority<ID>,
                    ) {
                        if let MovePriority::Second(.., shift) = priority {
                            *shift = Some(random.gen());
                        }
                        if queue.contains_key(priority) {
                            tie_break(queue, random, priority);
                        }
                    }

                    if queue.contains_key(&priority) {
                        tie_break(queue, random, &mut priority);
                    }

                    queue.insert(priority, Indexed(pokemon, action));
                }
            }
        }
    }
}
