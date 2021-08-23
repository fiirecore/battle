use core::fmt::Display;
use log::{info, warn};
use rand::Rng;

use pokedex::{
    item::{usage::ItemUsageKind, Itemdex},
    pokemon::Health,
};

use crate::{
    data::*,
    message::{ClientMessage, ServerMessage},
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

pub mod queue;
pub use queue::move_queue;

pub struct Battle<'d, ID: Sized + Copy + PartialEq + Ord + Display> {
    state: BattleState<ID>,

    data: BattleData,

    player1: BattlePlayer<'d, ID>,
    player2: BattlePlayer<'d, ID>,
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

impl<'d, ID: Sized + Copy + PartialEq + Ord + Display> Battle<'d, ID> {
    pub fn new(
        data: BattleData,
        player1: BattlePlayer<'d, ID>,
        player2: BattlePlayer<'d, ID>,
    ) -> Self {
        Self {
            state: BattleState::default(),
            data,
            player1,
            player2,
        }
    }

    pub fn data(&self) -> &BattleData {
        &self.data
    }

    pub fn begin(&mut self) {
        self.player1.party.reveal_active();
        self.player2.party.reveal_active();

        fn player_begin<ID: Copy>(
            data: BattleData,
            player: &mut BattlePlayer<ID>,
            other: &BattlePlayer<ID>,
        ) {
            player
                .endpoint
                .send(ServerMessage::Begin(ValidatedPlayer::new(
                    data, player, other,
                )));
        }

        player_begin(self.data.clone(), &mut self.player1, &self.player2);
        player_begin(self.data.clone(), &mut self.player2, &self.player1);

        self.state = BattleState::StartSelecting;
    }

    pub fn end(&mut self, winner: Option<ID>) {
        self.state = BattleState::End(winner);
        Self::set_winner(winner, &mut self.player1, &mut self.player2);
    }

    pub fn process(&mut self) {
        Self::receive(&mut self.player1, &mut self.player2);
        Self::receive(&mut self.player2, &mut self.player1);
    }

    fn receive(player: &mut BattlePlayer<'d, ID>, other: &mut BattlePlayer<'d, ID>) {
        while let Some(message) = player.endpoint.receive() {
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
                        player
                            .endpoint
                            .send(ServerMessage::ConfirmFaintReplace(active, can))
                    }

                    match player.party.active_contains(index) {
                        false => match player.party.pokemon.get(index) {
                            Some(pokemon) => match pokemon.fainted() {
                                false => {
                                    player.party.active[active] = Some(index.into());
                                    if let Some(pokemon) = player.party.know(index) {
                                        other.endpoint.send(ServerMessage::AddUnknown(
                                            index,
                                            pokemon.uninit(),
                                        ));
                                    }
                                    other.endpoint.send(ServerMessage::FaintReplace(
                                        PokemonIndex {
                                            team: player.party.id,
                                            index: active,
                                        },
                                        index,
                                    ));
                                    can(player, active, true)
                                }
                                true => can(player, active, false),
                            },
                            None => can(player, active, false),
                        },
                        true => can(player, active, false),
                    }
                }
                ClientMessage::Forfeit => Self::set_winner(Some(other.party.id), player, other),
                ClientMessage::LearnMove(pokemon, move_id, index) => {
                    if let Some(pokemon) = player.party.pokemon.get_mut(pokemon) {
                        if pokemon.learnable_moves.remove(&move_id) {
                            pokemon.replace_move(index as _, &move_id);
                        }
                    }
                }
                ClientMessage::FinishedTurnQueue => player.waiting = true,
            }
        }
    }

    pub fn update<R: Rng + Clone + 'static, E: MoveEngine>(
        &mut self,
        random: &mut R,
        engine: &mut E,
        itemdex: &'d Itemdex,
    ) {
        self.process();

        match self.state {
            BattleState::Start => self.begin(),
            BattleState::StartSelecting => {
                fn start_selecting<ID>(player: &mut BattlePlayer<ID>) {
                    player.waiting = false;
                    player.endpoint.send(ServerMessage::StartSelecting);
                }
                start_selecting(&mut self.player1);
                start_selecting(&mut self.player2);
                self.state = BattleState::WaitSelecting;
            }
            BattleState::WaitSelecting => {
                if self.player1.party.ready_to_move() && self.player2.party.ready_to_move() {
                    self.state = BattleState::StartMoving;
                }
            }
            BattleState::StartMoving => {
                let queue = move_queue(&mut self.player1.party, &mut self.player2.party, random);

                let player_queue = self.client_queue(random, engine, itemdex, queue);

                // end queue calculations

                self.player2
                    .endpoint
                    .send(ServerMessage::TurnQueue(player_queue.clone()));
                self.player1
                    .endpoint
                    .send(ServerMessage::TurnQueue(player_queue));

                self.state = BattleState::WaitMoving;
            }
            BattleState::WaitMoving => {
                if self.player1.waiting && self.player2.waiting {
                    if self.player2.party.all_fainted() {
                        info!("{} wins!", self.player1.name());
                        Self::set_winner(
                            Some(self.player1.party.id),
                            &mut self.player1,
                            &mut self.player2,
                        );
                    } else if self.player1.party.all_fainted() {
                        info!("{} wins!", self.player2.name());
                        Self::set_winner(
                            Some(self.player2.party.id),
                            &mut self.player1,
                            &mut self.player2,
                        );
                    } else if !self.player1.party.needs_replace()
                        && !self.player2.party.needs_replace()
                    {
                        self.state = BattleState::StartSelecting;
                    }
                }
            }
            BattleState::End(..) => (),
        }
    }

    fn set_winner(
        winner: Option<ID>,
        player1: &mut BattlePlayer<ID>,
        player2: &mut BattlePlayer<ID>,
    ) {
        player1.endpoint.send(ServerMessage::Winner(winner));
        player2.endpoint.send(ServerMessage::Winner(winner));
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, BattleState::End(..))
    }

    pub fn winner(&self) -> Option<Option<&ID>> {
        match &self.state {
            BattleState::End(.., winner) => Some(winner.as_ref()),
            _ => None,
        }
    }

    pub fn client_queue<R: Rng + Clone + 'static, E: MoveEngine>(
        &mut self,
        random: &mut R,
        engine: &mut E,
        itemdex: &'d Itemdex,
        queue: Vec<BoundAction<ID, BattleMove>>,
    ) -> Vec<BoundAction<ID, ClientMove>> {
        let mut player_queue = Vec::with_capacity(queue.len());

        for instance in queue {
            let (user, other) = match instance.pokemon.team == self.player1.party.id {
                true => (&mut self.player1, &mut self.player2),
                false => (&mut self.player2, &mut self.player1),
            };

            if let Some(user_pokemon) = user.party.active(instance.pokemon.index) {
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
                                    warn!(
                                            "Could not use move '{}' because it has no target implemented.",
                                            user_pokemon
                                                .moves
                                                .get(move_index)
                                                .map(|i| i.m.name.as_str())
                                                .unwrap_or("Unknown")
                                        );
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
                                        TargetLocation::User => Some(user_pokemon),
                                    }
                                    .map(|p| (location, p))
                                })
                                .collect();

                            user_pokemon.use_own_move(random, engine, move_index, targets)
                        };

                        if let Some((used_move, output)) = turn {
                            let mut actions = Vec::with_capacity(output.len());

                            for (location, action) in output {
                                let (user, other) =
                                    match instance.pokemon.team == self.player1.party.id {
                                        true => (&mut self.player1.party, &mut self.player2.party),
                                        false => (&mut self.player2.party, &mut self.player1.party),
                                    };

                                let (target_party, index) = match location {
                                    TargetLocation::Opponent(index) => (other, index),
                                    TargetLocation::User => (user, instance.pokemon.index),
                                    TargetLocation::Team(index) => (user, index),
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
                                            let user = match instance.pokemon.team
                                                == self.player1.party.id
                                            {
                                                true => &mut self.player1,
                                                false => &mut self.player2,
                                            };

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
                            match instance.pokemon.team == self.player1.party.id {
                                true => &mut self.player1.party,
                                false => &mut self.player2.party,
                            }
                            .active_mut(instance.pokemon.index)
                            .unwrap()
                            .moves[move_index]
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
                                                user.endpoint.send(ServerMessage::Catch(
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
                            other
                                .endpoint
                                .send(ServerMessage::AddUnknown(new, unknown.uninit()))
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
