use pokedex::{moves::MoveTarget, pokemon::{InitPokemon, Party}};
use rand::Rng;

use crate::{
    message::{ClientMessage, ServerMessage},
    moves::{
        client::{ClientAction, ClientMove},
        BattleMove, MoveTargetInstance,
    },
    player::{UninitRemotePlayer, PlayerKnowable},
    BattleEndpoint,
};

pub struct BattlePlayerAi<'d, R: Rng, ID: Default + PartialEq> {
    random: R,
    local: PlayerKnowable<ID, InitPokemon<'d>>,
    remote: UninitRemotePlayer<ID>,
    messages: Vec<ClientMessage>,
}

impl<'d, R: Rng, ID: Default + PartialEq> BattlePlayerAi<'d, R, ID> {
    pub fn new(random: R, party: Party<InitPokemon<'d>>) -> Self {
        let mut local = PlayerKnowable::default();
        local.party.pokemon = party;
        Self {
            random,
            local,
            remote: Default::default(),
            messages: Default::default(),
        }
    }
}

impl<'d, R: Rng, ID: Default + PartialEq> BattleEndpoint<ID> for BattlePlayerAi<'d, R, ID> {
    fn send(&mut self, message: ServerMessage<ID>) {
        match message {
            ServerMessage::Begin(validate) => {
                self.local.id = validate.id;
                self.local.name = validate.name;
                self.local.active = validate.active;
                self.remote = validate.remote;
            },
            ServerMessage::StartSelecting => {
                for (active, pokemon) in self.local.active_iter() {
                    // crashes when moves run out
                    let moves: Vec<usize> = pokemon
                        .moves
                        .iter()
                        .enumerate()
                        .filter(|(_, instance)| instance.pp != 0)
                        .map(|(index, _)| index)
                        .collect();

                    let move_index = moves[self.random.gen_range(0..moves.len())];

                    let target = match &pokemon.moves[move_index].m.target {
                        MoveTarget::Any => MoveTargetInstance::Any(
                            false,
                            self.random.gen_range(0..self.remote.active.len()),
                        ),
                        MoveTarget::Ally => {
                            let index = self.random.gen_range(1..self.local.active.len());
                            let index = if index >= active { index + 1 } else { index };
                            MoveTargetInstance::Ally(index)
                        }
                        MoveTarget::Allies => MoveTargetInstance::Allies,
                        MoveTarget::UserOrAlly => MoveTargetInstance::UserOrAlly(
                            self.random.gen_range(0..self.local.active.len()),
                        ),
                        MoveTarget::User => MoveTargetInstance::User,
                        MoveTarget::Opponent => MoveTargetInstance::Opponent(
                            self.random.gen_range(0..self.remote.active.len()),
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
                                        if instance.team == self.local.id {
                                            if let Some(pokemon) =
                                                self.local.active_mut(instance.index)
                                            {
                                                pokemon.hp = 0;
                                            }

                                            let available: Vec<usize> = self
                                                .local
                                                .pokemon
                                                .iter()
                                                .enumerate()
                                                .filter(|(index, pokemon)| {
                                                    !self
                                                        .local
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
                                                self.local.active[instance.index] = Some(r);

                                                self.messages.push(ClientMessage::ReplaceFaint(
                                                    instance.index,
                                                    r,
                                                ));
                                            }
                                        }
                                    }
                                    ClientAction::SetExp(.., level) => {
                                        match instance.pokemon.team == self.local.id {
                                            false => {
                                                if let Some(pokemon) = self
                                                    .remote
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
                                                    self.local.active_mut(instance.pokemon.index)
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
            ServerMessage::FaintReplace(pokemon, new) => if let Some(index) = match pokemon.team == self.local.id {
                true => &mut self.local.active,
                false => &mut self.remote.active,
            }.get_mut(pokemon.index) { *index = Some(new) },
            ServerMessage::AddUnknown(index, unknown) => self.remote.add_unknown(index, unknown),
            ServerMessage::Winner(..) | ServerMessage::Catch(..) => (),
            ServerMessage::ConfirmFaintReplace(index, can) => if !can {
                log::error!("AI cannot replace pokemon at active index {}", index)
            },
        }
    }

    fn receive(&mut self) -> Option<ClientMessage> {
        self.messages.pop()
    }
}
