use core::{fmt::Display, hash::Hash};
use log::{info, warn};
use rand::Rng;

use pokedex::{
    item::{usage::ItemUsageKind, Item},
    moves::owned::OwnedMove,
    Dex, Uninitializable,
};

use crate::{
    data::*,
    message::{ClientMessage, EndState, ServerMessage},
    moves::{
        damage::DamageResult,
        engine::{MoveEngine, MoveResult},
        BattleMove, ClientMove, ClientMoveAction,
    },
    party::PartyIndex,
    player::{BattlePlayer, ValidatedPlayer},
    pokemon::{battle::BattlePokemon, PokemonIndex},
    BattleEndpoint, Indexed,
};

mod collection;
pub use collection::*;

pub mod queue;
pub use queue::move_queue;

pub struct Battle<
    'd,
    ID: Display + Clone + Ord + Hash + 'static,
    E: BattleEndpoint<ID, AS>,
    const AS: usize,
> {
    state: BattleState,
    data: BattleData,
    players: collection::BattleMap<ID, BattlePlayer<'d, ID, E, AS>>, // can change to dex implementation, and impl Identifiable for BattlePlayer
}

#[derive(Debug)]
enum BattleState {
    Start,
    StartSelecting,
    WaitSelecting,
    StartMoving,
    WaitMoving,
    End,
}

impl Default for BattleState {
    fn default() -> Self {
        Self::Start
    }
}

impl<'d, ID: Display + Clone + Ord + Hash + 'static, E: BattleEndpoint<ID, AS>, const AS: usize>
    Battle<'d, ID, E, AS>
{
    pub fn new(
        data: BattleData,
        players: impl IntoIterator<Item = BattlePlayer<'d, ID, E, AS>>,
    ) -> Self {
        Self {
            state: BattleState::default(),
            data,
            players: players
                .into_iter()
                .map(|p| (p.party.id().clone(), p))
                .collect(),
        }
    }

    pub fn begin(&mut self) {
        for mut player in self.players.values_mut() {
            player.party.reveal_active();
        }

        for mut player in self.players.values_mut() {
            let v = ValidatedPlayer::new(self.data.clone(), &player, self.players.values());
            player.send(ServerMessage::Begin(v));
        }

        self.state = BattleState::StartSelecting;
    }

    pub fn end(&mut self, winner: Option<ID>) {
        self.state = BattleState::End;
        eprintln!("To - do: implement end of battle");
        for (id, mut player) in self.players.iter_mut() {
            match Some(id) == winner.as_ref() {
                true => player.send(ServerMessage::End(EndState::Win)),
                false => player.send(ServerMessage::End(EndState::Lose)),
            }
        }
    }

    pub fn update<R: Rng + Clone + 'static, ENG: MoveEngine>(
        &mut self,
        random: &mut R,
        engine: &mut ENG,
        itemdex: &'d impl Dex<Item>,
    ) {
        for (waiting, mut player) in self.players.values_waiting_mut() {
            while let Some(message) = player.receive() {
                match message {
                    ClientMessage::Move(active, bmove) => {
                        match player
                            .party
                            .active
                            .get_mut(active)
                            .map(Option::as_mut)
                            .flatten()
                        {
                            Some(pokemon) => {
                                pokemon.queued_move = Some(bmove);
                                // party.client.confirm_move(active);
                            }
                            None => warn!(
                                "Party {} could not add move #{} to pokemon #{}",
                                player.party.name(),
                                bmove,
                                active
                            ),
                        }
                    }
                    ClientMessage::ReplaceFaint(active, index) => {
                        match player.party.active_contains(index) {
                            false => match player.party.pokemon.get(index) {
                                Some(pokemon) => match pokemon.fainted() {
                                    false => {
                                        player.party.active[active] = Some(index.into());
                                        let unknown = player.party.know(index);
                                        for mut other in self.players.values_mut() {
                                            if let Some(pokemon) = unknown.as_ref() {
                                                other.send(ServerMessage::AddUnknown(
                                                    PokemonIndex(player.party.id().clone(), index),
                                                    pokemon.clone().uninit(),
                                                ));
                                            }
                                            other.send(ServerMessage::FaintReplace(
                                                PokemonIndex(player.party.id().clone(), active),
                                                index,
                                            ));
                                        }

                                        player
                                            .send(ServerMessage::ConfirmFaintReplace(active, true));
                                    }
                                    true => player
                                        .send(ServerMessage::ConfirmFaintReplace(active, false)),
                                },
                                None => {
                                    player.send(ServerMessage::ConfirmFaintReplace(active, false))
                                }
                            },
                            true => player.send(ServerMessage::ConfirmFaintReplace(active, false)),
                        }
                    }
                    ClientMessage::Forfeit => {
                        // self.remove(player.party.id);
                        // Self::set_winner(Some(other.party.id), player, other),
                    }
                    ClientMessage::LearnMove(pokemon, move_id, index) => {
                        if let Some(pokemon) = player.party.pokemon.get_mut(pokemon) {
                            if pokemon.learnable_moves.remove(&move_id) {
                                pokemon.moves.add(Some(index as _), &move_id);
                            }
                        }
                    }
                    ClientMessage::FinishedTurnQueue => waiting.set(true),
                }
            }
        }

        match self.state {
            BattleState::Start => self.begin(),
            BattleState::StartSelecting => {
                for (waiting, mut player) in self.players.values_waiting_mut() {
                    waiting.set(false);
                    player.send(ServerMessage::StartSelecting);
                }
                self.state = BattleState::WaitSelecting;
            }
            BattleState::WaitSelecting => {
                if self.players.values().all(|p| p.party.ready_to_move()) {
                    self.state = BattleState::StartMoving;
                }
            }
            BattleState::StartMoving => {
                let queue = move_queue(&mut self.players, random);

                let player_queue = self.client_queue(random, engine, itemdex, queue);

                // end queue calculations

                for mut player in self.players.values_mut() {
                    player.send(ServerMessage::TurnQueue(player_queue.clone()));
                }

                self.state = BattleState::WaitMoving;
            }
            BattleState::WaitMoving => {
                if self.players.all_waiting() {
                    for (id, player) in self.players.iter() {
                        if player.party.all_fainted() {
                            self.remove_player(id, EndState::Lose);
                        }
                    }

                    if self.players.len() >= 1 {
                        let winner = self.players.keys().next().cloned();
                        self.end(winner);
                        return;
                    } else if self.players.values().all(|p| !p.party.needs_replace()) {
                        self.state = BattleState::StartSelecting;
                    }
                }
            }
            BattleState::End => (),
        }
    }

    pub fn remove_player(&self, id: &ID, kind: EndState) {
        if let Some(mut player) = self.players.deactivate(id) {
            player.send(ServerMessage::End(kind));
        }
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, BattleState::End)
    }

    pub fn data(&self) -> &BattleData {
        &self.data
    }

    pub fn winner(&self) -> Option<ID> {
        match &self.state {
            BattleState::End => self.players.values().next().map(|p| p.id().clone()),
            _ => None,
        }
    }

    pub fn client_queue<R: Rng + Clone + 'static, ENG: MoveEngine>(
        &mut self,
        random: &mut R,
        engine: &mut ENG,
        itemdex: &'d impl Dex<Item>,
        queue: Vec<Indexed<ID, BattleMove<ID>>>,
    ) -> Vec<Indexed<ID, ClientMove<ID>>> {
        let mut player_queue = Vec::with_capacity(queue.len());

        for Indexed(instance, bmove) in queue {
            match bmove {
                BattleMove::Move(move_index, targets) => {                       

                    let turn = match self.players.get(instance.team()) {
                        Some(user) => {
                            match user.party.active(instance.index()) {
                                Some(pokemon) => {
                                    match pokemon
                                    .moves
                                    .get(move_index)
                                    .map(OwnedMove::try_use)
                                    .flatten() {
                                        Some(used_move) => engine.execute(random, used_move, (instance, pokemon), targets, &self.players).map_err(|err| {
                                            warn!("Cannot execute move {} for user {}'s pokemon {} with error {}", used_move.name, user.name(), pokemon.name(), err);
                                        }).ok(),
                                        None => {
                                            log::warn!("Cannot use move #{} for user {}'s {}", move_index, user.name(), pokemon.name());
                                            None
                                        },
                                    }
                                }
                                None => {
                                    log::warn!("Cannot get user {}'s pokemon at active slot {}", user.name(), instance.index());
                                    None
                                },
                            }
                        }
                        None => {
                            log::error!("Cannot get user {}!", instance.team());
                            None
                        },
                    };

                    if let Some(output) = turn {
                        let mut actions = Vec::with_capacity(output.len());

                        for (location, action) in output {
                            let mut user = self.players.get_mut(instance.pokemon.team()).unwrap();

                            if let (Some(mut target_user), index) = match &location {
                                TargetLocation::Opponent(id, index) => {
                                    (self.players.get_mut(id), *index)
                                }
                                TargetLocation::User => (Some(user), instance.0.index()),
                                TargetLocation::Team(index) => (Some(user), *index),
                            } {
                                if let Some(target) = target_user.party.active_mut(index) {
                                    fn on_damage<'d, ID>(
                                        location: TargetLocation<ID>,
                                        pokemon: &mut BattlePokemon<'d>,
                                        actions: &mut Vec<(TargetLocation<ID>, ClientMoveAction)>,
                                        result: DamageResult<Health>,
                                    ) {
                                        pokemon.hp = pokemon.hp.saturating_sub(result.damage);
                                        actions.push((
                                            location,
                                            ClientMoveAction::SetDamage(DamageResult {
                                                damage: pokemon.percent_hp(),
                                                effective: result.effective,
                                                crit: result.crit,
                                            }),
                                        ));
                                    }

                                    match action {
                                        MoveResult::Damage(result) => {
                                            on_damage(location, target, &mut actions, result)
                                        }
                                        MoveResult::Ailment(ailment) => {
                                            target.ailment = Some(ailment);
                                            actions.push((
                                                location,
                                                ClientMoveAction::Ailment(ailment),
                                            ));
                                        }
                                        MoveResult::Heal(health) => {
                                            let hp = health.abs() as u16;
                                            target.hp = match health.is_positive() {
                                                true => target.hp + hp.min(target.max_hp()),
                                                false => target.hp.saturating_sub(hp),
                                            };
                                            actions.push((
                                                location,
                                                ClientMoveAction::SetHP(target.percent_hp()),
                                            ));
                                        }
                                        MoveResult::Stat(stat, stage) => {
                                            target.stages.change_stage(stat, stage);
                                            actions.push((
                                                location,
                                                ClientMoveAction::AddStat(stat, stage),
                                            ));
                                        }
                                        MoveResult::Flinch => target.flinch = true,
                                        MoveResult::Miss => {
                                            actions.push((
                                                TargetLocation::User,
                                                ClientMoveAction::Miss,
                                            ));
                                            continue;
                                        }
                                        MoveResult::Error => actions
                                            .push((TargetLocation::User, ClientMoveAction::Error)),
                                    }

                                    if target.fainted() {
                                        let experience = target.exp_from();
                                        let experience =
                                            match matches!(self.data.type_, BattleType::Wild) {
                                                true => experience.saturating_mul(3) / 2,
                                                false => experience,
                                            };

                                        #[cfg(debug_assertions)]
                                        let experience = experience.saturating_mul(7);

                                        if let Some(active) =
                                            target_user.party.active.get_mut(index)
                                        {
                                            *active = None;
                                        }

                                        if target_user.id() != instance.pokemon.team() {
                                            drop(&mut target_user);

                                            let mut user = self
                                                .players
                                                .get_mut(instance.pokemon.team())
                                                .unwrap();

                                            if user.settings.gains_exp {
                                                let pokemon = user
                                                    .party
                                                    .active_mut(instance.0.index())
                                                    .unwrap();

                                                pokemon
                                                    .learnable_moves
                                                    .extend(pokemon.instance.add_exp(experience));

                                                actions.push((
                                                    TargetLocation::User,
                                                    ClientMoveAction::SetExp(
                                                        experience,
                                                        pokemon.level,
                                                    ),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // reduce PP by 1
                        user.party
                            .active_mut(instance.0.index())
                            .unwrap()
                            .moves
                            .get_mut(move_index)
                            .unwrap()
                            .decrement();

                        player_queue.push(BoundAction {
                            pokemon: instance.pokemon,
                            action: ClientMove::Move(used_move, actions),
                        });
                    }
                }
                BattleMove::UseItem(id, target) => match itemdex.try_get(&id) {
                    Some(item) => {
                        if match &item.usage.kind {
                            ItemUsageKind::Script | ItemUsageKind::Actions(..) => {
                                match user.party.active_mut(target.index()) {
                                    Some(pokemon) => pokemon.try_use_item(&item),
                                    None => false,
                                }
                            }
                            ItemUsageKind::Pokeball => match self.data.type_ {
                                BattleType::Wild => {
                                    if let Some(mut other) = self.players.get_mut(target.team()) {
                                        if let Some(active) = other
                                            .party
                                            .active
                                            .get_mut(target.index())
                                            .map(Option::take)
                                            .flatten()
                                        {
                                            if let Some(pokemon) =
                                                other.party.remove(active.index())
                                            {
                                                user.send(ServerMessage::Catch(
                                                    pokemon.instance.uninit(),
                                                ));

                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }
                                _ => {
                                    info!("Cannot use pokeballs in trainer battles!");
                                    false
                                }
                            },
                            ItemUsageKind::None => true,
                        } {
                            player_queue.push(BoundAction {
                                pokemon: instance.pokemon,
                                action: ClientMove::UseItem(item.id, target.index()),
                            });
                        }
                    }
                    None => warn!("Could not get item with id {}", id),
                },
                BattleMove::Switch(new) => {
                    user.party.replace(instance.0.index(), Some(new));
                    if let Some(unknown) = user
                        .party
                        .index(instance.0.index())
                        .map(|index| user.party.know(index))
                        .flatten()
                    {
                        let unknown = unknown.uninit();
                        for mut other in self.players.values_mut() {
                            other.send(ServerMessage::AddUnknown(
                                user.party.id().clone(),
                                new,
                                unknown.clone(),
                            ))
                        }
                    }
                    player_queue.push(BoundAction {
                        pokemon: instance.pokemon,
                        action: ClientMove::Switch(new),
                    });
                }
            }
        }
        player_queue
    }
}
