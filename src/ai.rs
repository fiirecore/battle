//! Basic Battle AI

use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

use rand::{prelude::IteratorRandom, Rng};

use pokedex::pokemon::{owned::OwnedPokemon, party::Party, Health};

use pokedex::{item::Item, moves::Move, pokemon::Pokemon, Dex};

use crate::{
    endpoint::{BattleEndpoint, ConnectionError, MpscClient, MpscEndpoint},
    message::{ClientMessage, ServerMessage},
    party::{PlayerParty, RemoteParty},
    pokemon::Indexed,
    select::*,
};

use hashbrown::HashMap;

pub struct BattleAi<ID: Eq + Hash + Clone, T> {
    local: Option<PlayerParty<ID, usize, OwnedPokemon, T>>,
    remotes: HashMap<ID, RemoteParty<ID, T>>,
    client: MpscClient<ID, T>,
    endpoint: MpscEndpoint<ID, T>,
}

#[derive(Debug)]
pub enum AiError {
    MissingMove,
    MissingPokemon(u32),
    MissingDex,
    OutOfPokemon,
    Connection(ConnectionError),
    SelectDenied,
}

impl<ID: Eq + Hash + Clone, T> BattleAi<ID, T> {
    const EXPECT_LOCAL: &'static str = "Could not get this AI's data!";

    pub fn new() -> Self {
        let (client, endpoint) = crate::endpoint::create();

        Self {
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

    pub fn active(&self) -> bool {
        self.local.is_some()
    }

    pub fn update(
        &mut self,
        random: &mut impl Rng,
        pokedex: &Dex<Pokemon>,
        movedex: &Dex<Move>,
        itemdex: &Dex<Item>,
    ) -> Result<(), AiError> {
        loop {
            match self.client.receive() {
                Ok(Some(message)) => {
                    let m = match message {
                        ServerMessage::PlayerData(_, local, _) => {
                            self.local = Some(PlayerParty {
                                id: local.id,
                                name: local.name,
                                active: local.active,
                                pokemon: local
                                    .pokemon
                                    .into_iter()
                                    .map(|p| {
                                        p.try_init(pokedex, movedex, itemdex)
                                            .expect("Unable to initialize AI pokemon!")
                                    })
                                    .collect(),
                                trainer: local.trainer,
                            });

                            Ok(())
                        }
                        ServerMessage::AddOpponent(remote) => {
                            self.remotes.insert(remote.id.clone(), remote);
                            Ok(())
                        }
                        ServerMessage::Select(index, select) => match select {
                            SelectMessage::Request(..) => self.queue_move(random, index),
                            SelectMessage::Confirm(confirm) => match confirm {
                                SelectConfirm::Move(id, sub) => {
                                    let local = self.local.as_mut().expect(Self::EXPECT_LOCAL);
                                    if let Some(pokemon) = local.active_mut(index) {
                                        if let Some(m) =
                                            pokemon.moves.iter_mut().find(|m| m.id() == &id)
                                        {
                                            m.pp = m.pp.saturating_sub(sub);
                                            Ok(())
                                        } else {
                                            Err(AiError::MissingMove)
                                        }
                                    } else {
                                        Err(AiError::MissingPokemon(line!()))
                                    }
                                }
                                SelectConfirm::Other => Ok(()),
                            },
                            SelectMessage::Deny => Err(AiError::SelectDenied),
                        },
                        ServerMessage::Results(actions) => {
                            let local = self.local.as_mut().expect(Self::EXPECT_LOCAL);
                            let mut user = None;
                            for action in actions {
                                match action {
                                    ClientAction::Announce(_, new_user, ..) => {
                                        user = new_user;
                                    }
                                    ClientAction::Actions(actions) => {
                                        for Indexed(target, action) in actions {
                                            match action {
                                                PublicAction::SetHP(hp) => {
                                                    let hp = hp.damage();
                                                    match target.team() == &local.id {
                                                        true => {
                                                            if let Some(pokemon) =
                                                                local.active_mut(target.index())
                                                            {
                                                                pokemon.hp = (hp
                                                                    * pokemon.max_hp() as f32)
                                                                    .ceil()
                                                                    as Health;
                                                                if pokemon.fainted() {
                                                                    match (0..local.pokemon.len())
                                                                        .into_iter()
                                                                        .filter(|i| {
                                                                            !local
                                                                                .active_contains(*i)
                                                                        })
                                                                        .choose(random)
                                                                    {
                                                                        Some(new) => {
                                                                            self.client.send(ClientMessage::Select(target.index(), BattleSelection::Pokemon(new)))?;
                                                                        }
                                                                        None => return Err(
                                                                            AiError::OutOfPokemon,
                                                                        ),
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        false => {
                                                            if let Some(pokemon) = self
                                                                .remotes
                                                                .get_mut(target.team())
                                                                .and_then(|party| {
                                                                    party.active_mut(target.index())
                                                                })
                                                                .and_then(Option::as_mut)
                                                            {
                                                                pokemon.hp = hp;
                                                            }
                                                        }
                                                    }
                                                }
                                                PublicAction::Switch(index) => {
                                                    let user = user.as_ref().expect(
                                                        "Expected a user when switching pokemon!",
                                                    );
                                                    match user.team() == &local.id {
                                                        true => {
                                                            local.replace(user.index(), Some(index))
                                                        }
                                                        false => {
                                                            if let Some(remote) =
                                                                self.remotes.get_mut(user.team())
                                                            {
                                                                remote.replace(
                                                                    user.index(),
                                                                    Some(index),
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                    }
                                    ClientAction::Error(..) => (),
                                }
                            }
                            Ok(())
                        }
                        ServerMessage::Replace(Indexed(target, new)) => {
                            let local = self.local.as_mut().expect(Self::EXPECT_LOCAL);
                            match target.team() == &local.id {
                                true => {
                                    local.replace(target.index(), Some(new));
                                }
                                false => {
                                    if let Some(remote) = self.remotes.get_mut(target.team()) {
                                        remote.replace(target.index(), Some(new));
                                    }
                                }
                            }
                            Ok(())
                        }
                        ServerMessage::Reveal(Indexed(target, unknown)) => {
                            if let Some(r) = self.remotes.get_mut(target.team()) {
                                match unknown {
                                    crate::pokemon::PokemonView::Partial(unknown) => {
                                        r.add(target.index(), Some(unknown))
                                    }
                                    crate::pokemon::PokemonView::Full(_) => (),
                                }
                            }
                            Ok(())
                        }
                        ServerMessage::Remove(id, ..) => {
                            if let Some(local) = self.local.as_mut() {
                                if local.id == id {
                                    self.stop_running();
                                }
                            }

                            self.remotes.remove(&id);
                            Ok(())
                        }
                        ServerMessage::End(..) => {
                            self.stop_running();
                            Ok(())
                        }
                    };
                    if m.is_err() {
                        return m;
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    self.stop_running();
                    return Err(err.into());
                }
            }
        }
        #[deprecated("temp workaround")]
        if let Some(local) = self.local.as_mut() {
            if let Some(fainted) = local.active_fainted() {
                println!("{:?}: fainted {}", local.name(), fainted);
                let new = (0..local.pokemon.len())
                    .into_iter()
                    .filter(|i| !local.active_contains(*i))
                    .choose(random)
                    .unwrap();
                self.client.send(ClientMessage::Select(
                    fainted,
                    BattleSelection::Pokemon(new),
                ));
            } else {
                let iter = local.active_iter().map(|(i, ..)| i ).collect::<Vec<_>>();
                for i in iter {
                    self.queue_move(random, i);
                }
            }
        }
        Ok(())
    }

    fn queue_move(&mut self, random: &mut impl Rng, active: usize) -> Result<(), AiError> {
        let local = self.local.as_mut().expect(Self::EXPECT_LOCAL);
        if let Some(pokemon) = local.active(active) {
            self.client
                .send(ClientMessage::Select(
                    active,
                    Self::pick_move(pokemon, random)?,
                ))
                .map_err(From::from)
        } else {
            Err(AiError::MissingPokemon(line!()))
        }
    }

    fn pick_move(
        pokemon: &OwnedPokemon,
        random: &mut impl Rng,
    ) -> Result<BattleSelection<ID>, AiError> {
        pokemon
            .moves
            .iter()
            .enumerate()
            .filter(|(_, instance)| !instance.is_empty())
            .map(|(.., m)| m.id())
            .choose(random)
            .cloned()
            .map(|index| BattleSelection::Move(index, None))
            .ok_or(AiError::MissingMove)
    }

    pub fn stop_running(&mut self) {
        self.local = None;
        self.remotes.clear();
    }
}

impl From<ConnectionError> for AiError {
    fn from(value: ConnectionError) -> Self {
        Self::Connection(value)
    }
}

impl<ID: Eq + Hash + Clone + Display, T: Debug> Display for BattleAi<ID, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.local.as_ref() {
            Some(local) => {
                write!(
                    f,
                    "AI #{}, Name: {}, Active: {:?}, Pokemon: [ ",
                    local.id,
                    local.name.as_deref().unwrap_or("N/A"),
                    local.active,
                )?;
                for (i, p) in local.pokemon.iter().enumerate() {
                    write!(
                        f,
                        "\"{}\": Lv. {} {}, HP {}/{}",
                        p.name(),
                        p.level,
                        p.pokemon.name,
                        p.hp(),
                        p.max_hp()
                    )?;
                    if i != local.pokemon.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, " ]")
            }
            None => write!(f, "Inactive AI"),
        }
    }
}
