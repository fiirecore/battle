use core::hash::Hash;
use log::{info, warn};
use rand::Rng;

use pokedex::{
    item::{usage::ItemUsageKind, Item},
    pokemon::Health,
    Dex, Identifiable, Uninitializable,
};

use crate::{
    data::*,
    message::{ClientMessage, EndState, ServerMessage},
    moves::{
        damage::DamageResult,
        engine::MoveEngine,
        target::{MoveTargetInstance, TargetLocation},
        BattleMove, ClientMove, ClientMoveAction, MoveResult,
    },
    player::{BattlePlayer, ValidatedPlayer},
    pokemon::{battle::BattlePokemon, PokemonIndex},
    BoundAction,
};

mod collection;

pub mod queue;
pub use queue::move_queue;

pub struct Battle<'d, ID: Copy + Ord + Hash> {
    state: BattleState<ID>,

    data: BattleData,

    pub players: collection::BattleMap<ID, BattlePlayer<'d, ID>>, // can change to dex implementation, and impl Identifiable for BattlePlayer
                                                              // player2: BattlePlayer<'d, ID>,
}

#[derive(Debug)]
pub enum BattleState<ID> {
    Start,
    StartSelecting,
    WaitSelecting,
    StartMoving,
    WaitMoving,
    End(Option<ID>),
}

impl<ID> Default for BattleState<ID> {
    fn default() -> Self {
        Self::Start
    }
}

impl<'d, ID: Copy + Ord + Hash> Battle<'d, ID> {
    pub fn new(data: BattleData, players: impl IntoIterator<Item = BattlePlayer<'d, ID>>) -> Self {
        Self {
            state: BattleState::default(),
            data,
            players: players.into_iter().map(|p| (p.party.id, p)).collect(),
        }
    }

    pub fn data(&self) -> &BattleData {
        &self.data
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
        self.state = BattleState::End(winner);
        for (id, mut player) in self.players.iter_mut() {
            match Some(id) == winner.as_ref() {
                true => player.send(ServerMessage::End(EndState::Win)),
                false => player.send(ServerMessage::End(EndState::Lose)),
            }
        }
    }

    pub fn process(&mut self) {
        for mut player in self.players.values_mut() {
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
                                "Party {} could not add move #{:?} to pokemon #{}",
                                player.name(),
                                bmove,
                                active
                            ),
                        }
                    }
                    ClientMessage::ReplaceFaint(active, index) => {
                        fn can<ID>(player: &mut BattlePlayer<ID>, active: usize, can: bool) {
                            player.send(ServerMessage::ConfirmFaintReplace(active, can))
                        }

                        match player.party.active_contains(index) {
                            false => match player.party.pokemon.get(index) {
                                Some(pokemon) => match pokemon.fainted() {
                                    false => {
                                        player.party.active[active] = Some(index.into());
                                        let unknown = player.party.know(index);
                                        for mut other in self.players.values_mut() {
                                            if let Some(pokemon) = unknown.as_ref() {
                                                other.send(ServerMessage::AddUnknown(
                                                    player.party.id,
                                                    index,
                                                    pokemon.clone().uninit(),
                                                ));
                                            }
                                            other.send(ServerMessage::FaintReplace(
                                                PokemonIndex {
                                                    team: player.party.id,
                                                    index: active,
                                                },
                                                index,
                                            ));
                                        }

                                        can(&mut player, active, true)
                                    }
                                    true => can(&mut player, active, false),
                                },
                                None => can(&mut player, active, false),
                            },
                            true => can(&mut player, active, false),
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
                    ClientMessage::FinishedTurnQueue => player.waiting = true,
                }
            }
        }
    }

    pub fn update<R: Rng + Clone + 'static, E: MoveEngine>(
        &mut self,
        random: &mut R,
        engine: &mut E,
        itemdex: &'d impl Dex<Item>,
    ) {
        self.process();

        match self.state {
            BattleState::Start => self.begin(),
            BattleState::StartSelecting => {
                for mut player in self.players.values_mut() {
                    player.waiting = false;
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
                if self.players.values().all(|p| p.waiting) {
                    for (id, player) in self.players.iter() {
                        if player.party.all_fainted() {
                            self.remove_player(id, EndState::Lose);
                        }
                    }

                    if self.players.len() >= 1 {
                        let winner = self.players.keys().next().copied();
                        self.end(winner);
                        return;
                    } else if self.players.values().all(|p| !p.party.needs_replace()) {
                        self.state = BattleState::StartSelecting;
                    }
                }
            }
            BattleState::End(..) => (),
        }
    }

    pub fn remove_player(&self, id: &ID, kind: EndState) {
        if let Some(mut player) = self.players.deactivate(id) {
            player.send(ServerMessage::End(kind));
        }
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, BattleState::End(..))
    }

    pub fn winner(&self) -> Option<Option<&ID>> {
        match &self.state {
            BattleState::End(winner) => Some(winner.as_ref()),
            _ => None,
        }
    }

    pub fn client_queue<R: Rng + Clone + 'static, E: MoveEngine>(
        &mut self,
        random: &mut R,
        engine: &mut E,
        itemdex: &'d impl Dex<Item>,
        queue: Vec<BoundAction<ID, BattleMove>>,
    ) -> Vec<BoundAction<ID, ClientMove>> {
        let mut player_queue = Vec::with_capacity(queue.len());

        for instance in queue {
            let mut user = self.players.get_mut(&instance.pokemon.team);
            let user = user.as_deref_mut();

            let other = self.players.values_mut().next();

            if let (Some(user), Some(mut other)) = (user, other) {
                match instance.action {
                    BattleMove::Move(move_index, target) => {
                        let turn = {
                            let targets: Vec<TargetLocation> = match target {
                                MoveTargetInstance::Any(user, index) => match user {
                                    true => match index == instance.pokemon.index {
                                        true => TargetLocation::user().collect(),
                                        false => TargetLocation::team(index).collect(),
                                    },
                                    false => TargetLocation::opponent(index).collect(),
                                },
                                MoveTargetInstance::Ally(index) => {
                                    TargetLocation::team(index).collect()
                                }
                                MoveTargetInstance::Allies => TargetLocation::allies(
                                    instance.pokemon.index,
                                    user.party.active.len(),
                                )
                                .collect(),
                                MoveTargetInstance::UserOrAlly(index) => {
                                    match index == instance.pokemon.index {
                                        true => TargetLocation::user().collect(),
                                        false => TargetLocation::team(index).collect(),
                                    }
                                }
                                MoveTargetInstance::User => TargetLocation::user().collect(),
                                MoveTargetInstance::Opponent(index) => {
                                    TargetLocation::opponent(index).collect()
                                }
                                MoveTargetInstance::AllOpponents => {
                                    TargetLocation::opponents(other.party.active.len()).collect()
                                }
                                MoveTargetInstance::RandomOpponent => TargetLocation::opponent(
                                    random.gen_range(0..other.party.active.len()),
                                )
                                .collect(),
                                MoveTargetInstance::AllOtherPokemon => {
                                    TargetLocation::all_other_pokemon(
                                        instance.pokemon.index,
                                        user.party.active.len(),
                                        other.party.active.len(),
                                    )
                                    .collect()
                                }
                                MoveTargetInstance::None => {
                                    if let Some(user) = user.party.active(instance.pokemon.index) {
                                        warn!(
                                            "Could not use move '{}' because it has no target implemented.",
                                            user
                                                .moves
                                                .get(move_index)
                                                .map(|i| i.0.name())
                                                .unwrap_or("Unknown")
                                        );
                                    }
                                    vec![]
                                }
                                MoveTargetInstance::UserAndAllies => {
                                    TargetLocation::user_and_allies(
                                        instance.pokemon.index,
                                        user.party.active.len(),
                                    )
                                    .collect()
                                }
                                MoveTargetInstance::AllPokemon => TargetLocation::all_pokemon(
                                    instance.pokemon.index,
                                    user.party.active.len(),
                                    other.party.active.len(),
                                )
                                .collect(),
                            };

                            let targets = targets
                                .into_iter()
                                .flat_map(|location| {
                                    match location {
                                        TargetLocation::Opponent(index) => {
                                            other.party.active(index)
                                        }
                                        TargetLocation::Team(index) => user.party.active(index),
                                        TargetLocation::User => {
                                            user.party.active(instance.pokemon.index)
                                        }
                                    }
                                    .map(|p| (location, p))
                                })
                                .collect();

                            user.party
                                .active(instance.pokemon.index)
                                .map(|p| p.use_own_move(random, engine, move_index, targets))
                                .unwrap_or_else(|| {
                                    warn!(
                                        "User {} cannot use move for pokemon #{}",
                                        user.name(),
                                        instance.pokemon.index
                                    );
                                    Default::default()
                                })
                        };

                        if let Some((used_move, output)) = turn {
                            let mut actions = Vec::with_capacity(output.len());

                            for (location, action) in output {

                                let (target_party, index) = match location {
                                    TargetLocation::Opponent(index) => (&mut other.party, index),
                                    TargetLocation::User => {
                                        (&mut user.party, instance.pokemon.index)
                                    }
                                    TargetLocation::Team(index) => (&mut user.party, index),
                                };

                                if let Some(target) = target_party.active_mut(index) {
                                    fn on_damage<'d>(
                                        location: TargetLocation,
                                        pokemon: &mut BattlePokemon<'d>,
                                        actions: &mut Vec<(TargetLocation, ClientMoveAction)>,
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

                                        if let Some(active) = target_party.active.get_mut(index) {
                                            *active = None;
                                        }

                                        if target_party.id != instance.pokemon.team {
                                            if user.settings.gains_exp {
                                                let pokemon = &mut user
                                                    .party
                                                    .active_mut(instance.pokemon.index)
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

                            // reduce PP by 1
                            user.party
                                .active_mut(instance.pokemon.index)
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
                                    match user.party.active_mut(target) {
                                        Some(pokemon) => pokemon.try_use_item(&item),
                                        None => false,
                                    }
                                }
                                ItemUsageKind::Pokeball => match self.data.type_ {
                                    BattleType::Wild => {
                                        if let Some(active) = other
                                            .party
                                            .active
                                            .get_mut(target)
                                            .map(Option::take)
                                            .flatten()
                                        {
                                            if let Some(pokemon) = other.party.remove(active.index)
                                            {
                                                user.send(ServerMessage::Catch(
                                                    pokemon.instance.uninit(),
                                                ));
                                            }
                                        }
                                        true
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
                                    action: ClientMove::UseItem(item.id, target),
                                });
                            }
                        }
                        None => warn!("Could not get item with id {}", id),
                    },
                    BattleMove::Switch(new) => {
                        user.party.replace(instance.pokemon.index, Some(new));
                        if let Some(unknown) = user
                            .party
                            .index(instance.pokemon.index)
                            .map(|index| user.party.know(index))
                            .flatten()
                        {
                            other.send(ServerMessage::AddUnknown(
                                user.party.id,
                                new,
                                unknown.uninit(),
                            ))
                        }
                        player_queue.push(BoundAction {
                            pokemon: instance.pokemon,
                            action: ClientMove::Switch(new),
                        });
                    }
                }
            }
        }
        player_queue
    }
}

impl<'d, ID: Copy + Ord + Hash + core::fmt::Debug> core::fmt::Debug for Battle<'d, ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Battle").field("state", &self.state).field("data", &self.data).field("players", &self.players).finish()
    }
}