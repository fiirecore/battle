use pokedex::moves::MoveTarget;
use rand::Rng;

use crate::{
    message::{ClientMessage, ServerMessage},
    moves::{
        client::{ClientAction, ClientMove},
        BattleMove, MoveTargetInstance,
    },
    player::{LocalPlayer, RemotePlayer},
    BattleEndpoint,
};

pub struct BattlePlayerAi<R: Rng, ID: Default + PartialEq> {
    random: R,
    player: LocalPlayer<ID>,
    opponent: RemotePlayer<ID>,
    messages: Vec<ClientMessage>,
}

impl<R: Rng, ID: Default + PartialEq> BattlePlayerAi<R, ID> {
    pub fn new(random: R) -> Self {
        Self {
            random,
            player: LocalPlayer::default(),
            opponent: RemotePlayer::default(),
            messages: Default::default(),
        }
    }
}

impl<R: Rng, ID: Default + PartialEq> BattleEndpoint<ID> for BattlePlayerAi<R, ID> {
    fn send(&mut self, message: ServerMessage<ID>) {
        match message {
            ServerMessage::User(_, player) => self.player = player,
            ServerMessage::Opponents(opponent) => self.opponent = opponent,
            ServerMessage::StartSelecting => {
                for (active, pokemon) in self.player.active_iter() {
                    // crashes when moves run out
                    let moves: Vec<usize> = pokemon
                        .moves
                        .iter()
                        .enumerate()
                        .filter(|(_, instance)| instance.pp != 0)
                        .map(|(index, _)| index)
                        .collect();

                    let move_index = moves[self.random.gen_range(0..moves.len())];

                    let target = match &pokemon.moves[move_index].move_ref.target {
                        MoveTarget::Any => MoveTargetInstance::Any(
                            false,
                            self.random.gen_range(0..self.opponent.active.len()),
                        ),
                        MoveTarget::Ally => {
                            let index = self.random.gen_range(1..self.player.active.len());
                            let index = if index >= active { index + 1 } else { index };
                            MoveTargetInstance::Ally(index)
                        }
                        MoveTarget::Allies => MoveTargetInstance::Allies,
                        MoveTarget::UserOrAlly => MoveTargetInstance::UserOrAlly(
                            self.random.gen_range(0..self.player.active.len()),
                        ),
                        MoveTarget::User => MoveTargetInstance::User,
                        MoveTarget::Opponent => MoveTargetInstance::Opponent(
                            self.random.gen_range(0..self.opponent.active.len()),
                        ),
                        MoveTarget::AllOpponents => MoveTargetInstance::AllOpponents,
                        MoveTarget::RandomOpponent => MoveTargetInstance::RandomOpponent,
                        MoveTarget::AllOtherPokemon => MoveTargetInstance::AllOtherPokemon,
                        MoveTarget::Todo => MoveTargetInstance::Todo,
                        MoveTarget::UserAndAllies => MoveTargetInstance::UserAndAllies,
                        MoveTarget::AllPokemon => MoveTargetInstance::AllPokemon,
                    };

                    self.messages.push(ClientMessage::Move(
                        active,
                        BattleMove::Move(move_index, target),
                    ));
                }
            }

            ServerMessage::TurnQueue(actions) => {
                for instance in actions {
                    if let ClientMove::Move(.., instances) = &instance.action {
                        for actions in instances {
                            for moves in &actions.actions {
                                match moves {
                                    ClientAction::Faint(instance) => {
                                        if instance.team == self.player.id {
                                            if let Some(pokemon) =
                                                self.player.active_mut(instance.index)
                                            {
                                                pokemon.current_hp = 0;
                                            }

                                            let available: Vec<usize> = self
                                                .player
                                                .pokemon
                                                .iter()
                                                .enumerate()
                                                .filter(|(index, pokemon)| {
                                                    !self
                                                        .player
                                                        .active
                                                        .iter()
                                                        .any(|u| u == &Some(*index))
                                                        && !pokemon.fainted()
                                                })
                                                .map(|(index, _)| index)
                                                .collect(); // To - do: use position()

                                            if !available.is_empty() {
                                                let r = available
                                                    [self.random.gen_range(0..available.len())];
                                                self.player.active[instance.index] = Some(r);

                                                self.messages.push(ClientMessage::FaintReplace(
                                                    instance.index,
                                                    r,
                                                ));
                                            }
                                        }
                                    }
                                    ClientAction::SetExp(.., level) => {
                                        match instance.pokemon.team == self.player.id {
                                            false => {
                                                if let Some(pokemon) = self
                                                    .opponent
                                                    .active_mut(instance.pokemon.index)
                                                    .map(Option::as_mut)
                                                    .flatten()
                                                {
                                                    pokemon.level = *level;
                                                    // Ai does not learn moves
                                                }
                                            }
                                            true => {
                                                if let Some(pokemon) =
                                                    self.player.active_mut(instance.pokemon.index)
                                                {
                                                    pokemon.level = *level;
                                                }
                                            }
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                }
                self.messages.push(ClientMessage::FinishedTurnQueue);
            }
            ServerMessage::FaintReplace(pokemon, new) => match pokemon.team == self.player.id {
                true => self.player.active[pokemon.index] = new,
                false => self.opponent.active[pokemon.index] = new,
            },
            ServerMessage::AddUnknown(index, unknown) => self.opponent.add_unknown(index, unknown),
            ServerMessage::PokemonRequest(index, instance) => {
                self.opponent.add_instance(index, instance)
            }
            ServerMessage::CanFaintReplace(index, can) => {
                if !can {
                    log::error!("AI cannot replace fainted pokemon at {}", index);
                }
            }
            ServerMessage::Winner(..) | ServerMessage::PartyRequest(..) => (),
        }
    }

    fn receive(&mut self) -> Option<ClientMessage> {
        self.messages.pop()
    }
}
