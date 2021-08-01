use core::{fmt::Display, ops::Deref};
use log::{info, warn};
use rand::Rng;

use pokedex::{
    id::Dex,
    item::{ItemUseType, Itemdex},
    moves::{
        usage::{script::Engine, DamageResult, MoveResult, NoHitResult},
        Movedex,
    },
    pokemon::{Health, PokemonInstance},
    types::Effective,
};

use crate::{
    data::*,
    message::{ClientMessage, ServerMessage},
    moves::{
        client::{BoundClientMove, ClientAction, ClientActions, ClientMove},
        BattleMove, BoundBattleMove, MoveTargetInstance, MoveTargetLocation,
    },
    player::BattlePlayer,
    pokemon::PokemonIndex,
    state::BattleState,
};

pub struct Battle<ID: Sized + Copy + PartialEq + Ord + Display> {
    ////////////// if using hashmap, only remaining player should be winner
    state: BattleState<ID>,

    data: BattleData,

    player1: BattlePlayer<ID>,
    player2: BattlePlayer<ID>,
}

impl<ID: Sized + Copy + PartialEq + Ord + Display> Battle<ID> {
    pub fn new(data: BattleData, player1: BattlePlayer<ID>, player2: BattlePlayer<ID>) -> Self {
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

        fn player_begin<ID: Copy>(data: BattleData, player: &mut BattlePlayer<ID>, other: &BattlePlayer<ID>) {
            player.endpoint.send(ServerMessage::User(
                data,
                player.as_local(),
            ));
            player.endpoint.send(ServerMessage::Opponents(other.as_remote()));
        }

        player_begin(self.data.clone(), &mut self.player1, &self.player2);
        player_begin(self.data.clone(), &mut self.player2, &self.player1);

        self.state = BattleState::StartSelecting;
    }

    fn receive(data: &BattleData, player: &mut BattlePlayer<ID>, other: &mut BattlePlayer<ID>) {
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
                ClientMessage::FaintReplace(active, index) => {

                    fn can<ID>(player: &mut BattlePlayer<ID>, active: usize, can: bool) {
                        player.endpoint.send(ServerMessage::CanFaintReplace(active, can))
                    }

                    match player.party.active_contains(index) {
                        false => match player.party.pokemon.get(index) {
                            Some(pokemon) => match pokemon.fainted() {
                                false => {
                                    player.party.active[active] = Some(index.into());
                                    if let Some(pokemon) = player.party.know(index) {
                                        other
                                            .endpoint
                                            .send(ServerMessage::AddUnknown(index, pokemon));
                                    }
                                    other.endpoint.send(ServerMessage::FaintReplace(
                                        PokemonIndex {
                                            team: player.party.id,
                                            index: active,
                                        },
                                        Some(index),
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
                ClientMessage::RequestPokemon(request) => {
                    if let Some(pokemon) = other.party.pokemon.get(request) {
                        if matches!(data.type_, BattleType::Wild)
                            || pokemon.requestable
                            || (pokemon.fainted() && pokemon.known)
                        {
                            player.endpoint.send(ServerMessage::PokemonRequest(
                                request,
                                pokemon.deref().clone(),
                            ));
                        }
                    }
                }
                ClientMessage::Forfeit => Self::set_winner(other.party.id, player, other),
                ClientMessage::AddLearnedMove(pokemon, index, move_id) => {
                    if let Some(pokemon) = player.party.pokemon.get_mut(pokemon) {
                        if pokemon.learnable_moves.contains(&move_id) {
                            if let Some(move_ref) = Movedex::try_get(&move_id) {
                                pokemon.replace_move(index, move_ref);
                            }
                        }
                    }
                }
                ClientMessage::FinishedTurnQueue => player.waiting = true,
            }
        }
    }

    pub fn update<R: Rng + Clone + 'static>(&mut self, random: &mut R, engine: &Engine) {
        
        Self::receive(&self.data, &mut self.player1, &mut self.player2);
        Self::receive(&self.data, &mut self.player2, &mut self.player1);

        match self.state {
            BattleState::Setup => self.begin(),

            BattleState::StartWait => (),
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
                let queue =
                    crate::moves::move_queue(&mut self.player1.party, &mut self.player2.party);

                let player_queue = self.client_queue(random, engine, queue);

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
                            self.player1.party.id,
                            &mut self.player1,
                            &mut self.player2,
                        );
                    } else if self.player1.party.all_fainted() {
                        info!("{} wins!", self.player2.name());
                        Self::set_winner(
                            self.player2.party.id,
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

    fn set_winner(winner: ID, player1: &mut BattlePlayer<ID>, player2: &mut BattlePlayer<ID>) {
        player1.endpoint.send(ServerMessage::Winner(winner));
        player2.endpoint.send(ServerMessage::Winner(winner));
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, BattleState::End(..))
    }

    pub fn winner(&self) -> Option<&ID> {
        match &self.state {
            BattleState::End(.., winner) => Some(winner),
            _ => None,
        }
    }

    pub fn client_queue<R: Rng + Clone + 'static>(
        &mut self,
        random: &mut R,
        engine: &Engine,
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
                                        .map(|m| m.move_ref.name.as_str())
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

                        let turn = user_pokemon.use_own_move(random, engine, move_index, targets);

                        let mut results = Vec::with_capacity(turn.1.len());

                        for (location, result) in turn.1 {
                            let mut actions = Vec::new();

                            {
                                let user = match instance.pokemon.team == self.player1.party.id {
                                    true => &mut self.player1.party,
                                    false => &mut self.player2.party,
                                };

                                if let Some(user) = user.active_mut(instance.pokemon.index) {
                                    for result in &result {
                                        match result {
                                            MoveResult::Drain(.., heal) => {
                                                let gain = heal.is_positive();
                                                let heal = (heal.abs() as u16).min(user.base.hp());
                                                user.current_hp = match gain {
                                                    true => user.current_hp.saturating_add(heal),
                                                    false => user.current_hp.saturating_sub(heal),
                                                };
                                                actions
                                                    .push(ClientAction::UserHP(user.percent_hp()));
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                            }

                            for result in result {
                                let (user, other) =
                                    match instance.pokemon.team == self.player1.party.id {
                                        true => (&mut self.player1.party, &mut self.player2.party),
                                        false => (&mut self.player2.party, &mut self.player1.party),
                                    };

                                let (target_party, index) = match location {
                                    MoveTargetLocation::Opponent(index) => (other, index),
                                    MoveTargetLocation::User => (user, instance.pokemon.index),
                                    MoveTargetLocation::Team(index) => (user, index),
                                };

                                if let Some(target) = target_party.active_mut(index) {
                                    fn on_damage<ID>(
                                        pokemon: &mut PokemonInstance,
                                        actions: &mut Vec<ClientAction<ID>>,
                                        result: DamageResult<Health>,
                                    ) {
                                        pokemon.current_hp =
                                            pokemon.current_hp.saturating_sub(result.damage);
                                        actions.push(ClientAction::TargetHP(
                                            pokemon.hp() as f32 / pokemon.max_hp() as f32,
                                            result.crit,
                                        ));
                                        if !matches!(result.effective, Effective::Effective) {
                                            actions.push(ClientAction::Effective(result.effective));
                                        }
                                    }

                                    match result {
                                        MoveResult::Damage(result) => {
                                            on_damage(target, &mut actions, result)
                                        }
                                        MoveResult::Status(effect) => {
                                            target.effect = Some(effect);
                                            actions.push(ClientAction::Status(effect));
                                        }
                                        MoveResult::Drain(result, ..) => {
                                            on_damage(target, &mut actions, result)
                                        }
                                        MoveResult::StatStage(stat) => {
                                            target.base.change_stage(stat);
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
                                            NoHitResult::Todo => actions.push(ClientAction::Fail),
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

                        player_queue.push(BoundClientMove {
                            pokemon: instance.pokemon,
                            action: ClientMove::Move(turn.0, results),
                        });
                    }
                    BattleMove::UseItem(id, target) => match Itemdex::try_get(&id) {
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
                                                user.endpoint.send(ServerMessage::PokemonRequest(
                                                    active.index,
                                                    pokemon.deref().clone(),
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
                                    action: ClientMove::UseItem(item, target),
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
                            other.endpoint.send(ServerMessage::AddUnknown(new, unknown))
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
