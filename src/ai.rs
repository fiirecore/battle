//! Basic Battle AI

use core::hash::Hash;

use rand::{prelude::IteratorRandom, Rng};

use pokedex::pokemon::{owned::OwnedPokemon, party::Party, Health};

use pokedex::{item::Item, moves::Move, pokemon::Pokemon, Dex, Identifiable};

use crate::{
    endpoint::{BattleEndpoint, MpscClient, MpscEndpoint},
    message::{ClientMessage, ServerMessage},
    party::{PlayerParty, RemoteParty},
    pokemon::{Indexed, TeamIndex},
    select::*,
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

pub enum AiError {}

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

    pub fn update(
        &mut self,
        pokedex: &Dex<Pokemon>,
        movedex: &Dex<Move>,
        itemdex: &Dex<Item>,
    ) -> Result<(), AiError> {
        loop {
            match self.client.receive() {
                Ok(Some(message)) => match message {
                    ServerMessage::PlayerData(data, local, bag) => {
                        let mut party = Party::new();

                        for pokemon in local.pokemon {
                            // maybe substitute for try_init
                            match pokemon.init(&mut self.random, pokedex, movedex, itemdex) {
                                Some(pokemon) => party.push(pokemon),
                                None => {}
                            }
                        }

                        self.running = true;

                        self.local = Some(PlayerParty {
                            id: local.id,
                            name: local.name,
                            active: local.active,
                            pokemon: party,
                            trainer: local.trainer,
                        });
                    }
                    ServerMessage::AddOpponent(remote) => {
                        self.remotes.insert(remote.id.clone(), remote);
                    }
                    ServerMessage::Select(index, select) => match select {
                        SelectMessage::Request(positions) => self.queue_move(index),
                        SelectMessage::Confirm(confirm) => match confirm {
                            SelectConfirm::Move(id, sub) => {
                                if let Some(local) = self.local.as_mut() {
                                    if let Some(pokemon) = local.active_mut(index) {
                                        if let Some(m) =
                                            pokemon.moves.iter_mut().find(|m| m.id() == &id)
                                        {
                                            m.1 = m.1.saturating_sub(sub);
                                        }
                                    }
                                }
                            }
                            SelectConfirm::Other => (),
                        },
                    },
                    ServerMessage::Results(actions) => {
                        if let Some(local) = self.local.as_mut() {
                            let mut user = None;
                            for action in actions {
                                match action {
                                    ClientAction::Announce(_, new_user, ..) => {
                                        user = new_user;
                                    }
                                    ClientAction::Actions(actions) => {
                                        for Indexed(target_id, action) in actions {
                                            match action {
                                                ClientMoveAction::SetHP(hp) => {
                                                    let hp = hp.damage();
                                                    match user.as_ref().map(TeamIndex::team)
                                                        == Some(&local.id)
                                                    {
                                                        true => {
                                                            if let Some(pokemon) =
                                                                local.active_mut(target_id.index())
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
                                                                .and_then(|party| {
                                                                    party.active_mut(
                                                                        target_id.index(),
                                                                    )
                                                                })
                                                                .and_then(Option::as_mut)
                                                            {
                                                                pokemon.hp = hp;
                                                            }
                                                        }
                                                    }
                                                }
                                                ClientMoveAction::Switch(index) => {
                                                    if let Some(user) = user.as_ref() {
                                                        match user.team() == &local.id {
                                                            true => local
                                                                .replace(user.index(), Some(index)),
                                                            false => {
                                                                if let Some(remote) = self
                                                                    .remotes
                                                                    .get_mut(user.team())
                                                                {
                                                                    remote.replace(
                                                                        user.index(),
                                                                        Some(index),
                                                                    );
                                                                }
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
                        } else {
                            self.forfeit();
                        }
                    }
                    ServerMessage::Replace(Indexed(target, new)) => {
                        if let Some(index) = self
                            .local
                            .as_mut()
                            .filter(|local| target.team() == &local.id)
                            .and_then(|party| party.active.get_mut(target.index()))
                            .or_else(|| {
                                self.remotes
                                    .values_mut()
                                    .find(|r| &r.id == target.team())
                                    .and_then(|party| party.active.get_mut(target.index()))
                            })
                        {
                            *index = Some(new);
                        }
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
                    }
                    ServerMessage::Remove(id, ..) => {
                        if let Some(local) = self.local.as_mut() {
                            if local.id == id {
                                self.stop_running();
                            }
                        }

                        self.remotes.remove(&id);
                    }
                    ServerMessage::PlayerData(data, party, bag) => {
                        self.local = Some(PlayerParty {
                            id: party.id,
                            name: party.name,
                            active: party.active,
                            pokemon: party
                                .pokemon
                                .into_iter()
                                .map(|p| p.try_init(pokedex, movedex, itemdex).unwrap())
                                .collect(),
                            trainer: party.trainer,
                        })
                    }
                    ServerMessage::AddOpponent(remote) => {
                        self.remotes.insert(remote.id.clone(), remote);
                    }
                    ServerMessage::End(winner) => self.stop_running(),
                },
                Ok(None) => break,
                Err(err) => {
                    self.stop_running();
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
        Ok(())
    }

    fn queue_move(&mut self, active: usize) {
        if let Some(local) = self.local.as_mut() {
            if let Some(pokemon) = local.active(active) {
                self.client.send(ClientMessage::Select(
                    active,
                    Self::pick_move(active, pokemon, &mut self.random),
                ));
            } else {
                self.forfeit();
                return;
            }
        }
    }

    #[deprecated(note = "return errors/try to fix instead of using internally")]
    pub fn forfeit(&mut self) {
        self.client.send(ClientMessage::TryForfeit);
        self.stop_running();
    }

    fn pick_move(active: usize, pokemon: &OwnedPokemon, random: &mut R) -> BattleSelection<ID> {
        let index = pokemon
            .moves
            .iter()
            .enumerate()
            .filter(|(_, instance)| !instance.is_empty())
            .map(|(.., m)| m.id())
            .choose(random)
            .cloned()
            .unwrap_or(Move::UNKNOWN);

        BattleSelection::Move(index, None)
    }

    pub fn stop_running(&mut self) {
        self.running = false;
        self.remotes.clear();
    }
}
