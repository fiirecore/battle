use core::{fmt::Display, ops::Deref};
use log::{info, warn};
use rand::Rng;

use pokedex::{
    item::{ItemUseType, Itemdex},
    moves::{
        script::MoveEngine,
        usage::{DamageResult, MoveResult, NoHitResult},
        MoveInstance, Movedex,
    },
    pokemon::{Health, InitPokemon},
    types::Effective,
};

use crate::{
    data::*,
    message::{ClientMessage, ServerMessage},
    moves::{
        client::{BoundClientMove, ClientAction, ClientActions, ClientMove},
        BattleMove, BoundBattleMove, MoveTargetInstance, MoveTargetLocation,
    },
    player::{BattlePlayer, ValidatedPlayer},
    pokemon::PokemonIndex,
};

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

    pub fn process(&mut self, movedex: &'d Movedex) {
        Self::receive(movedex, &mut self.player1, &mut self.player2);
        Self::receive(movedex, &mut self.player2, &mut self.player1);
    }

    fn receive(
        movedex: &'d Movedex,
        player: &mut BattlePlayer<'d, ID>,
        other: &mut BattlePlayer<'d, ID>,
    ) {
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
                            if let Some(move_ref) = movedex.try_get(&move_id) {
                                pokemon.replace_move(index as _, MoveInstance::new(move_ref));
                            }
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
        movedex: &'d Movedex,
        itemdex: &'d Itemdex,
    ) {
        self.process(movedex);

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
                let queue = crate::moves::move_queue(
                    &mut self.player1.party,
                    &mut self.player2.party,
                    random,
                );

                let player_queue = self.client_queue(random, engine, movedex, itemdex, queue);

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
        movedex: &'d Movedex,
        itemdex: &'d Itemdex,
        queue: Vec<BoundBattleMove<ID>>,
    ) -> Vec<BoundClientMove<ID>> {
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
                            let targets = match target {
                                MoveTargetInstance::Any(user, index) => vec![match user {
                                    true => match index == instance.pokemon.index {
                                        true => MoveTargetLocation::User,
                                        false => MoveTargetLocation::Team(index),
                                    },
                                    false => MoveTargetLocation::Opponent(index),
                                }],
                                MoveTargetInstance::Ally(index) => {
                                    vec![MoveTargetLocation::Team(index)]
                                }
                                MoveTargetInstance::Allies => MoveTargetLocation::allies(
                                    instance.pokemon.index,
                                    user.party.active.len(),
                                ),
                                MoveTargetInstance::UserOrAlly(index) => {
                                    vec![match index == instance.pokemon.index {
                                        true => MoveTargetLocation::User,
                                        false => MoveTargetLocation::Team(index),
                                    }]
                                }
                                MoveTargetInstance::User => vec![MoveTargetLocation::User],
                                MoveTargetInstance::Opponent(index) => {
                                    vec![MoveTargetLocation::Opponent(index)]
                                }
                                MoveTargetInstance::AllOpponents => {
                                    MoveTargetLocation::opponents(other.party.active.len())
                                }
                                MoveTargetInstance::RandomOpponent => {
                                    vec![MoveTargetLocation::Opponent(
                                        random.gen_range(0..other.party.active.len()),
                                    )]
                                }
                                MoveTargetInstance::AllOtherPokemon => {
                                    MoveTargetLocation::all_other_pokemon(
                                        instance.pokemon.index,
                                        user.party.active.len(),
                                        other.party.active.len(),
                                    )
                                }
                                MoveTargetInstance::Todo => {
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
                                    MoveTargetLocation::user_and_allies(
                                        instance.pokemon.index,
                                        user.party.active.len(),
                                    )
                                }
                                MoveTargetInstance::AllPokemon => MoveTargetLocation::all_pokemon(
                                    instance.pokemon.index,
                                    user.party.active.len(),
                                    other.party.active.len(),
                                ),
                            };

                            let targets = targets
                                .into_iter()
                                .flat_map(|target| {
                                    match target {
                                        MoveTargetLocation::Opponent(index) => {
                                            other.party.active(index)
                                        }
                                        MoveTargetLocation::Team(index) => user.party.active(index),
                                        MoveTargetLocation::User => Some(user_pokemon),
                                    }
                                    .map(|i| (target, i.deref()))
                                })
                                .collect();

                            user_pokemon.use_own_move(random, engine, move_index, targets)
                        };

                        if let Some((used_move, output)) = turn {
                            let mut results = Vec::default(); ////Vec::with_capacity(output.len());

                            for (location, result) in output {
                                let mut actions = Vec::new();

                                {
                                    let user = match instance.pokemon.team == self.player1.party.id
                                    {
                                        true => &mut self.player1.party,
                                        false => &mut self.player2.party,
                                    };

                                    if let Some(user) = user.active_mut(instance.pokemon.index) {
                                        for result in &result {
                                            match result {
                                                MoveResult::Drain(.., heal) => {
                                                    let gain = heal.is_positive();
                                                    let heal =
                                                        (heal.abs() as u16).min(user.max_hp());
                                                    user.hp = match gain {
                                                        true => user.hp.saturating_add(heal),
                                                        false => user.hp.saturating_sub(heal),
                                                    };
                                                    actions.push(ClientAction::UserHP(
                                                        user.percent_hp(),
                                                    ));
                                                }
                                                _ => (),
                                            }
                                        }
                                    }
                                }

                                for result in result {
                                    let (user, other) = match instance.pokemon.team
                                        == self.player1.party.id
                                    {
                                        true => (&mut self.player1.party, &mut self.player2.party),
                                        false => (&mut self.player2.party, &mut self.player1.party),
                                    };

                                    let (target_party, index) = match location {
                                        MoveTargetLocation::Opponent(index) => (other, index),
                                        MoveTargetLocation::User => (user, instance.pokemon.index),
                                        MoveTargetLocation::Team(index) => (user, index),
                                    };

                                    if let Some(target) = target_party.active_mut(index) {
                                        fn on_damage<'d, ID>(
                                            pokemon: &mut InitPokemon<'d>,
                                            actions: &mut Vec<ClientAction<ID>>,
                                            result: DamageResult<Health>,
                                        ) {
                                            pokemon.hp = pokemon.hp.saturating_sub(result.damage);
                                            actions.push(ClientAction::TargetHP(
                                                pokemon.percent_hp(),
                                                result.crit,
                                            ));
                                            if !matches!(result.effective, Effective::Effective) {
                                                actions.push(ClientAction::Effective(
                                                    result.effective,
                                                ));
                                            }
                                        }

                                        match result {
                                            MoveResult::Damage(result) => {
                                                on_damage(target, &mut actions, result)
                                            }
                                            MoveResult::Status(ailment) => {
                                                target.ailment = Some(ailment);
                                                actions.push(ClientAction::Ailment(ailment));
                                            }
                                            MoveResult::Drain(result, ..) => {
                                                on_damage(target, &mut actions, result)
                                            }
                                            MoveResult::StatStage(stat) => {
                                                target.stages.change_stage(stat);
                                                actions.push(ClientAction::StatStage(stat));
                                            }
                                            MoveResult::Flinch => target.flinch = true,
                                            MoveResult::NoHit(result) => match result {
                                                NoHitResult::Ineffective => actions.push(
                                                    ClientAction::Effective(Effective::Ineffective),
                                                ),
                                                NoHitResult::Miss => {
                                                    actions.push(ClientAction::Miss);
                                                    continue;
                                                }
                                                NoHitResult::Todo | NoHitResult::Error => {
                                                    actions.push(ClientAction::Fail)
                                                }
                                            },
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

                                            actions.push(ClientAction::Faint(PokemonIndex {
                                                team: target_party.id,
                                                index,
                                            }));

                                            if let Some(active) = target_party.active.get_mut(index)
                                            {
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

                                                    pokemon.learnable_moves.extend(
                                                        pokemon
                                                            .instance
                                                            .add_exp(movedex, experience),
                                                    );

                                                    actions.push(ClientAction::SetExp(
                                                        experience,
                                                        pokemon.level,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }

                                results.push(ClientActions { location, actions });
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

                            player_queue.push(BoundClientMove {
                                pokemon: instance.pokemon,
                                action: ClientMove::Move(used_move, results),
                            });
                        }
                    }
                    BattleMove::UseItem(id, target) => match itemdex.try_get(&id) {
                        Some(item) => {
                            if match &item.usage {
                                ItemUseType::Script(script) => {
                                    match user.party.active_mut(target) {
                                        Some(pokemon) => {
                                            pokemon.execute_item_script(script);
                                            true
                                        }
                                        None => false,
                                    }
                                }
                                ItemUseType::Pokeball => match self.data.type_ {
                                    BattleType::Wild => {
                                        if let Some(active) = other
                                            .party
                                            .active
                                            .get_mut(target)
                                            .map(Option::take)
                                            .flatten()
                                        {
                                            if other.party.pokemon.len() > active.index {
                                                let mut pokemon =
                                                    other.party.pokemon.remove(active.index);
                                                pokemon.caught = true;
                                                user.endpoint.send(ServerMessage::Catch(
                                                    pokemon.deref().clone().into(),
                                                ));
                                                if let Err(err) =
                                                    user.party.pokemon.try_push(pokemon)
                                                {
                                                    warn!("Could not catch pokemon because the player's party has reached maximum capacity and PCs have not been implemented.");
                                                    warn!("Error: {}", err);
                                                }
                                            }
                                        }
                                        true
                                    }
                                    _ => {
                                        info!("Cannot use pokeballs in trainer battles!");
                                        false
                                    }
                                },
                                ItemUseType::None => true,
                            } {
                                player_queue.push(BoundClientMove {
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
                        player_queue.push(BoundClientMove {
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
