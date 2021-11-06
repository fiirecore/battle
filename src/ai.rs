//! Basic Battle AI

use core::{hash::Hash, ops::Deref};

use rand::{prelude::IteratorRandom, Rng};

use pokedex::pokemon::{owned::OwnedPokemon, party::Party, Health};

use pokedex::{pokemon::Pokemon, moves::Move, item::Item};

use crate::{
    endpoint::{MpscClient, MpscEndpoint},
    message::{ClientMessage, FailedAction, ServerMessage, StartableAction, TimedAction},
    moves::{BattleMove, ClientMove, ClientMoveAction},
    party::{PlayerParty, RemoteParty},
    pokemon::Indexed,
};

#[derive(Clone)]
pub struct BattleAi<
    R: Rng,
    ID: Eq + Hash + Clone,
    P: Deref<Target = Pokemon>,
    M: Deref<Target = Move>,
    I: Deref<Target = Item>,
> {
    random: R,
    local: PlayerParty<ID, usize, OwnedPokemon<P, M, I>>,
    remotes: hashbrown::HashMap<ID, RemoteParty<ID>>,
    client: MpscClient<ID>,
    endpoint: MpscEndpoint<ID>,
    finished: bool,
}

impl<R: Rng, ID: Eq + Hash + Clone, 
P: Deref<Target = Pokemon>,
M: Deref<Target = Move>,
I: Deref<Target = Item>,> BattleAi<R, ID, P, M, I> {
    pub fn new(temp_id: ID, random: R, active: usize, party: Party<OwnedPokemon<P, M, I>>) -> Self {
        let (client, endpoint) = crate::endpoint::create();

        Self {
            random,
            local: PlayerParty::new(temp_id, None, active, party),
            remotes: Default::default(),
            client,
            endpoint,
            finished: false,
        }
    }

    pub fn party(&self) -> &Party<OwnedPokemon<P, M, I>> {
        &self.local.pokemon
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn endpoint(&self) -> &MpscEndpoint<ID> {
        &self.endpoint
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
                    ServerMessage::Replace(Indexed(target, new)) => {
                        if let Some(index) = match target.team() == self.local.id() {
                            true => Some(&mut self.local.active),
                            false => self
                                .remotes
                                .values_mut()
                                .filter(|r| r.id() == target.team())
                                .map(|r| &mut r.active)
                                .next(),
                        }
                        .map(|a| a.get_mut(target.index()))
                        .flatten()
                        {
                            *index = Some(new)
                        }
                    }
                    ServerMessage::AddRemote(Indexed(target, unknown)) => {
                        if let Some(r) = self.remotes.get_mut(target.team()) {
                            r.add(target.index(), Some(unknown));
                        }
                    }
                    ServerMessage::Catch(..) => (),
                    ServerMessage::Fail(action) => match action {
                        FailedAction::Replace(active) => {
                            log::error!(
                                "AI {} cannot replace pokemon at active index {}",
                                self.local.name(),
                                active
                            );
                            self.client.send(ClientMessage::Forfeit);
                        }
                        FailedAction::Move(active) => {
                            if let Some(pokemon) = self.local.active(active) {
                                Self::queue_move(active, pokemon, &mut self.random, &self.client);
                            } else {
                                log::error!(
                                    "AI {} cannot use move for pokemon at active index {}",
                                    self.local.name(),
                                    active
                                );
                            }
                        }
                        FailedAction::Switch(active) => {
                            log::error!(
                                "AI {} cannot switch pokemon at active index {}",
                                self.local.name(),
                                active
                            );
                            self.client.send(ClientMessage::Forfeit);
                        }
                    },
                    ServerMessage::PlayerEnd(..) | ServerMessage::GameEnd(..) => {
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
            Self::queue_move(active, pokemon, &mut self.random, &self.client);
        }
    }

    fn queue_move(
        active: usize,
        pokemon: &OwnedPokemon<P, M, I>,
        random: &mut R,
        client: &MpscClient<ID>,
    ) {
        let index = pokemon
            .moves
            .iter()
            .enumerate()
            .filter(|(_, instance)| !instance.is_empty())
            .map(|(index, ..)| index)
            .choose(random)
            .unwrap_or(0);

        client.send(ClientMessage::Move(active, BattleMove::Move(index, None)));
    }
}
