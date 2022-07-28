//! Basic Battle AI

use core::hash::Hash;

use rand::{prelude::IteratorRandom, Rng};

use pokedex::pokemon::{owned::OwnedPokemon, party::Party, Health};

use pokedex::{item::Item, moves::Move, pokemon::Pokemon, Dex};

use crate::{
    endpoint::{MpscClient, MpscEndpoint},
    message::{ClientMessage, FailedAction, ServerMessage, StartableAction, TimedAction},
    moves::{BattleMove, ClientMove, ClientMoveAction},
    party::{PlayerParty, RemoteParty},
    pokemon::Indexed,
    prelude::CommandAction,
};

use hashbrown::HashMap;

pub struct BattleAi<ID: Eq + Hash + Clone, R: Rng, T> {
    running: bool,
    random: R,
    local: Option<PlayerParty<ID, usize, OwnedPokemon, T>>,
    remotes: HashMap<ID, RemoteParty<ID, T>>,
    client: MpscClient<ID, T>,
    endpoint: MpscEndpoint<ID, T>,
}

impl<ID: Eq + Hash + Clone, R: Rng, T> BattleAi<ID, R, T> {
    pub fn new(random: R) -> Self {
        let (client, endpoint) = crate::endpoint::create();

        Self {
            running: false,
            random,
            local: None,
            remotes: Default::default(),
            client,
            endpoint,
        }
    }

    pub fn party(&self) -> Option<&Party<OwnedPokemon>> {
        self.local.as_ref().map(|local| &local.pokemon)
    }

    pub fn endpoint(&self) -> &MpscEndpoint<ID, T> {
        &self.endpoint
    }

    pub fn update(&mut self, pokedex: &Dex<Pokemon>, movedex: &Dex<Move>, itemdex: &Dex<Item>) {
        while !self.client.receiver.is_empty() {
            match self.client.receiver.try_recv() {
                Ok(message) => match message {
                    ServerMessage::Begin(validate) => {
                        let mut party = Party::new();

                        for pokemon in validate.local.pokemon {
                            // maybe substitute for try_init
                            match pokemon.init(&mut self.random, pokedex, movedex, itemdex) {
                                Some(pokemon) => party.push(pokemon),
                                None => {
                                    log::error!(
                                        "AI {:?} unable to initialize party",
                                        validate.local.name
                                    );
                                    self.forfeit();
                                    return;
                                }
                            }
                        }

                        self.running = true;

                        self.local = Some(PlayerParty {
                            id: validate.local.id,
                            name: validate.local.name,
                            active: validate.local.active,
                            pokemon: party,
                            trainer: validate.local.trainer,
                        });

                        self.remotes = validate
                            .remotes
                            .into_iter()
                            .map(|p| (p.id().clone(), p))
                            .collect();
                    }
                    message => {
                        if self.running {
                            match message {
                                ServerMessage::Begin(..) => unreachable!(),
                                ServerMessage::PlayerEnd(id, ..) => {
                                    if Some(&id) == self.local.as_ref().map(|l| &l.id) {
                                        self.running = false;
                                    }
                                }
                                ServerMessage::GameEnd(..) => {
                                    self.running = false;
                                }
                                other => match self.local.as_mut() {
                                    Some(local) => match other {
                                        ServerMessage::Start(a) => match a {
                                            StartableAction::Selecting => Self::queue_moves(
                                                local,
                                                &mut self.random,
                                                &self.client,
                                            ),
                                            StartableAction::Turns(actions) => {
                                                for Indexed(.., m) in actions {
                                                    if let ClientMove::Move(.., instances) = m {
                                                        for Indexed(target_id, action) in instances
                                                        {
                                                            match action {
                                                                ClientMoveAction::SetHP(hp) => {
                                                                    let hp = hp.damage();
                                                                    match target_id.team()
                                                                        == local.id()
                                                                    {
                                                                        true => {
                                                                            if let Some(pokemon) =
                                                                                local.active_mut(
                                                                                    target_id
                                                                                        .index(),
                                                                                )
                                                                            {
                                                                                pokemon.hp = (hp
                                                                                    * pokemon
                                                                                        .max_hp()
                                                                                        as f32)
                                                                                    .ceil()
                                                                                    as Health;
                                                                            }
                                                                        }
                                                                        false => {
                                                                            if let Some(pokemon) = self
                                                                            .remotes
                                                                            .get_mut(target_id.team()).and_then(|party| {
                                                                                party.active_mut(
                                                                                    target_id.index(),
                                                                                )
                                                                            }).and_then(Option::as_mut)
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
                                        ServerMessage::Replace(Indexed(target, new)) => {
                                            if let Some(index) = match target.team() == local.id() {
                                                true => Some(&mut local.active),
                                                false => self
                                                    .remotes
                                                    .values_mut()
                                                    .filter(|r| r.id() == target.team())
                                                    .map(|r| &mut r.active)
                                                    .next(),
                                            }
                                            .and_then(|a| a.get_mut(target.index()))
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

                                        ServerMessage::Ping(a) => match a {
                                            TimedAction::Selecting => {
                                                log::error!(
                                                    "AI {} unable to queue moves!",
                                                    local.name()
                                                );
                                                return;
                                            }
                                            TimedAction::Replace => {
                                                log::error!(
                                                    "AI {} Unable to replace pokemon!",
                                                    local.name()
                                                );
                                                self.forfeit();
                                                return;
                                            }
                                        },
                                        ServerMessage::Fail(action) => match action {
                                            FailedAction::Replace(active) => {
                                                log::error!(
                                                    "AI {} cannot replace pokemon at active index {}",
                                                    local.name(),
                                                    active
                                                );
                                                self.client.send(ClientMessage::Forfeit);
                                            }
                                            FailedAction::Move(active) => {
                                                if let Some(pokemon) = local.active(active) {
                                                    Self::queue_move(
                                                        active,
                                                        pokemon,
                                                        &mut self.random,
                                                        &self.client,
                                                    );
                                                } else {
                                                    log::error!("AI {} cannot use move for pokemon at active index {}", 
                                                local.name(),
                                                active);
                                                    self.forfeit();
                                                }
                                            }
                                            FailedAction::Switch(active) => {
                                                log::error!(
                                                    "AI {} cannot switch pokemon at active index {}",
                                                    local.name(),
                                                    active
                                                );
                                                self.client.send(ClientMessage::Forfeit);
                                            }
                                        },
                                        ServerMessage::Command(command) => match command {
                                            CommandAction::Faint(id) => {
                                                match id.team() == local.id() {
                                                    true => {
                                                        if let Some(pokemon) =
                                                            local.pokemon.get_mut(id.index())
                                                        {
                                                            pokemon.hp = 0;
                                                        }
                                                    }
                                                    false => {
                                                        if let Some(pokemon) = self
                                                            .remotes
                                                            .get_mut(id.team())
                                                            .and_then(|team| {
                                                                team.pokemon
                                                                    .get_mut(id.index())
                                                                    .map(Option::as_mut)
                                                            })
                                                            .flatten()
                                                        {
                                                            pokemon.hp = 0.0;
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        ServerMessage::Begin(..)
                                        | ServerMessage::PlayerEnd(..)
                                        | ServerMessage::GameEnd(..) => unreachable!(),
                                    },
                                    None => {
                                        log::error!("AI unable to get own player!");
                                        self.forfeit();
                                    }
                                },
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("Unable to receive server message with error {}", err);
                    self.forfeit();
                }
            }
        }

        if let Some(local) = self.local.as_mut() {
            while let Some(active) = local.active_fainted() {
                let new = local
                    .remaining()
                    .map(|(index, ..)| index)
                    .choose(&mut self.random);

                local.replace(active, new);

                if let Some(index) = new {
                    self.client.send(ClientMessage::ReplaceFaint(active, index));
                }
            }
        }
    }

    fn queue_moves(
        local: &mut PlayerParty<ID, usize, OwnedPokemon, T>,
        random: &mut R,
        client: &MpscClient<ID, T>,
    ) {
        for (active, pokemon) in local.active_iter() {
            Self::queue_move(active, pokemon, random, client);
        }
    }

    pub fn forfeit(&mut self) {
        self.client.send(ClientMessage::Forfeit);
        self.running = false;
    }

    fn queue_move(
        active: usize,
        pokemon: &OwnedPokemon,
        random: &mut R,
        client: &MpscClient<ID, T>,
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
