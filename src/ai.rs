use core::hash::Hash;

use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};

use rand::{prelude::IteratorRandom, Rng};

use pokedex::pokemon::{owned::OwnedPokemon, party::Party, Health};

use crate::{
    endpoint::{BattleEndpoint, ReceiveError},
    message::{ClientMessage, FailedAction, ServerMessage, StartableAction, TimedAction},
    moves::{BattleMove, ClientMove, ClientMoveAction},
    party::{PlayerParty, RemoteParty},
    pokemon::Indexed,
};

#[derive(Clone)]
pub struct BattleAi<'d, R: Rng, ID: Default + Eq + Hash + Clone, const AS: usize> {
    random: R,
    local: PlayerParty<ID, usize, OwnedPokemon<'d>, AS>,
    remotes: hashbrown::HashMap<ID, RemoteParty<ID, AS>>,
    client: BattleAiClient<ID, AS>,
    endpoint: BattleAiEndpoint<ID, AS>,
    finished: bool,
}

#[derive(Clone)]
struct BattleAiClient<ID, const AS: usize> {
    sender: Sender<ClientMessage<ID>>,
    receiver: Receiver<ServerMessage<ID, AS>>,
}

#[derive(Clone)]
pub struct BattleAiEndpoint<ID, const AS: usize> {
    pub receiver: Receiver<ClientMessage<ID>>,
    pub sender: Sender<ServerMessage<ID, AS>>,
}

impl<'d, R: Rng, ID: Default + Eq + Hash + Clone, const AS: usize> BattleAi<'d, R, ID, AS> {
    pub fn new(random: R, party: Party<OwnedPokemon<'d>>) -> Self {
        let (serv_sender, receiver) = unbounded();
        let (sender, serv_receiver) = unbounded();

        Self {
            random,
            local: PlayerParty::new(Default::default(), None, party),
            remotes: Default::default(),
            client: BattleAiClient { sender, receiver },
            endpoint: BattleAiEndpoint {
                receiver: serv_receiver,
                sender: serv_sender,
            },
            finished: false,
        }
    }

    pub fn party(&self) -> &Party<OwnedPokemon<'d>> {
        &self.local.pokemon
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn endpoint(&self) -> BattleAiEndpoint<ID, AS> {
        self.endpoint.clone()
    }

    pub fn update(&mut self) {
        while !self.client.receiver.is_empty() {
            match self.client.receiver.try_recv() {
                Ok(message) => match message {
                    ServerMessage::Begin(validate) => {
                        self.local.id = validate.id;
                        self.local.name = validate.name;
                        self.local.active = validate.active;
                        self.remotes = validate
                            .remotes
                            .into_iter()
                            .map(|p| (p.id().clone(), p))
                            .collect();
                    }
                    ServerMessage::Start(a) => match a {
                        StartableAction::Selecting => self.queue_moves(),
                        StartableAction::Turns(actions) => {
                            for Indexed(.., m) in actions {
                                if let ClientMove::Move(.., instances) = m {
                                    for Indexed(target_id, action) in instances {
                                        match action {
                                            ClientMoveAction::SetHP(hp) => {
                                                let hp = hp.damage();
                                                match target_id.team() == self.local.id() {
                                                    true => {
                                                        if let Some(pokemon) =
                                                            self.local.active_mut(target_id.index())
                                                        {
                                                            pokemon.hp = (hp
                                                                * pokemon.max_hp() as f32)
                                                                .ceil()
                                                                as Health;
                                                        }
                                                    }
                                                    false => {
                                                        if let Some(pokemon) = self
                                                            .remotes
                                                            .get_mut(target_id.team())
                                                            .map(|party| {
                                                                party.active_mut(target_id.index())
                                                            })
                                                            .flatten()
                                                            .map(Option::as_mut)
                                                            .flatten()
                                                        {
                                                            pokemon.hp = hp;
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
                    },
                    ServerMessage::Ping(a) => match a {
                        TimedAction::Selecting => {
                            log::warn!(
                                "AI {} was unable to queue moves, forfeiting...",
                                self.local.name()
                            );
                            self.client.send(ClientMessage::Forfeit);
                        }
                        TimedAction::Replace => {
                            log::warn!(
                                "AI {} was unable to replace pokemon, forfeiting...",
                                self.local.name()
                            );
                            log::warn!("AI {}: {}", self.local.name(), self.local.needs_replace());
                            self.client.send(ClientMessage::Forfeit);
                        }
                    },
                    ServerMessage::Replace(pokemon, new) => {
                        if let Some(index) = match pokemon.team() == self.local.id() {
                            true => Some(&mut self.local.active),
                            false => self
                                .remotes
                                .values_mut()
                                .filter(|r| r.id() == pokemon.team())
                                .map(|r| &mut r.active)
                                .next(),
                        }
                        .map(|a| a.get_mut(pokemon.index()))
                        .flatten()
                        {
                            *index = Some(new)
                        }
                    }
                    ServerMessage::AddRemote(index, unknown) => {
                        if let Some(r) = self.remotes.get_mut(index.team()) {
                            r.add(index.index(), Some(unknown));
                        }
                    }
                    ServerMessage::Catch(..) => (),
                    ServerMessage::Fail(action) => match action {
                        FailedAction::FaintReplace(index) => {
                            log::error!(
                                "AI {} cannot replace pokemon at active index {}",
                                self.local.name(),
                                index
                            );
                            self.client.send(ClientMessage::Forfeit);
                        }
                    },
                    ServerMessage::End => {
                        self.finished = true;
                    }
                },
                Err(err) => log::error!(
                    "AI at {} could not receive server message with error {}",
                    self.local.name(),
                    err
                ),
            }
        }

        while let Some(active) = self.local.active_fainted() {
            let new = self
                .local
                .remaining()
                .map(|(index, ..)| index)
                .choose(&mut self.random);

            self.local.replace(active, new);

            if let Some(index) = new {
                self.client.send(ClientMessage::ReplaceFaint(active, index));
            }
        }
    }

    fn queue_moves(&mut self) {
        for (active, pokemon) in self.local.active_iter() {
            let index = pokemon
                .moves
                .iter()
                .enumerate()
                .filter(|(_, instance)| instance.uses() != 0)
                .map(|(index, ..)| index)
                .choose(&mut self.random)
                .unwrap_or(0);

            self.client
                .send(ClientMessage::Move(active, BattleMove::Move(index, None)));
        }
    }
}

impl<ID, const AS: usize> BattleAiClient<ID, AS> {
    pub fn send(&self, message: ClientMessage<ID>) {
        if let Err(err) = self.sender.try_send(message) {
            log::error!("AI cannot send client message with error {}", err);
        }
    }
}

impl<ID, const AS: usize> BattleEndpoint<ID, AS> for BattleAiEndpoint<ID, AS> {
    fn send(&mut self, message: ServerMessage<ID, AS>) {
        if let Err(err) = self.sender.try_send(message) {
            log::error!("Cannot send server message to AI with error {}", err);
        }
    }

    fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>> {
        match self.receiver.try_recv() {
            Ok(m) => Ok(m),
            Err(err) => Err(match err {
                TryRecvError::Empty => None,
                TryRecvError::Disconnected => Some(ReceiveError::Disconnected),
            }),
        }
    }
}
