use core::hash::Hash;

use rand::{prelude::IteratorRandom, Rng};

use pokedex::pokemon::{owned::OwnedPokemon, party::Party, Health};

use crate::{
    message::{ClientMessage, ServerMessage},
    moves::{damage::DamageResult, BattleMove, ClientMove, ClientMoveAction},
    party::{PlayerParty, RemoteParty},
    BattleEndpoint, Indexed,
};

#[derive(Clone)]
pub struct BattleAi<'d, R: Rng, ID: Default + Eq + Hash + Clone, const AS: usize> {
    random: R,
    local: PlayerParty<ID, usize, OwnedPokemon<'d>, AS>,
    remotes: hashbrown::HashMap<ID, RemoteParty<ID, AS>>,
    messages: Vec<ClientMessage<ID>>,
}

impl<'d, R: Rng, ID: Default + Eq + Hash + Clone, const AS: usize> BattleAi<'d, R, ID, AS> {
    pub fn new(random: R, party: Party<OwnedPokemon<'d>>) -> Self {
        Self {
            random,
            local: PlayerParty::new(Default::default(), None, party),
            remotes: Default::default(),
            messages: Default::default(),
        }
    }

    pub fn party(&self) -> &Party<OwnedPokemon<'d>> {
        &self.local.pokemon
    }
}

impl<'d, R: Rng, ID: Default + Eq + Hash + Clone, const AS: usize> BattleEndpoint<ID, AS>
    for BattleAi<'d, R, ID, AS>
{
    fn send(&mut self, message: ServerMessage<ID, AS>) {
        match message {
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
            ServerMessage::StartSelecting => {
                for (active, pokemon) in self.local.active_iter() {
                    let index = pokemon
                        .moves
                        .iter()
                        .enumerate()
                        .filter(|(_, instance)| instance.uses() != 0)
                        .map(|(index, ..)| index)
                        .choose(&mut self.random)
                        .unwrap_or(0);

                    log::trace!(
                        "AI {}'s {} used {}!",
                        self.local.name(),
                        pokemon.name(),
                        pokemon.moves.get(index).unwrap().0.name
                    );

                    self.messages
                        .push(ClientMessage::Move(active, BattleMove::Move(index, None)));
                }
            }

            ServerMessage::TurnQueue(actions) => {
                for Indexed(.., m) in actions {
                    if let ClientMove::Move(.., instances) = m {
                        for Indexed(target_id, action) in instances {
                            match action {
                                ClientMoveAction::SetDamage(DamageResult {
                                    damage: hp, ..
                                })
                                | ClientMoveAction::SetHP(hp) => {
                                    match target_id.team() == self.local.id() {
                                        true => {
                                            if let Some(pokemon) =
                                                self.local.active_mut(target_id.index())
                                            {
                                                pokemon.hp =
                                                    (hp * pokemon.max_hp() as f32) as Health;

                                                if pokemon.hp == 0 {
                                                    log::trace!(
                                                        "AI {}'s pokemon at {:?} fainted!",
                                                        self.local.name(),
                                                        self.local.index(target_id.index())
                                                    );

                                                    let index = self
                                                        .local
                                                        .remaining()
                                                        .map(|(index, ..)| index)
                                                        .choose(&mut self.random);

                                                    self.local.replace(target_id.index(), index);

                                                    if let Some(index) = index {
                                                        self.messages.push(
                                                            ClientMessage::ReplaceFaint(
                                                                target_id.index(),
                                                                index,
                                                            ),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                        false => {
                                            if let Some(pokemon) = self
                                                .remotes
                                                .get_mut(target_id.team())
                                                .map(|party| party.active_mut(target_id.index()))
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
                self.messages.push(ClientMessage::FinishedTurnQueue);
            }
            ServerMessage::FaintReplace(pokemon, new) => {
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
            ServerMessage::End(..) | ServerMessage::Catch(..) => (),
            ServerMessage::ConfirmFaintReplace(index, can) => {
                if !can {
                    log::error!("AI cannot replace pokemon at active index {}", index)
                }
            }
        }
    }

    fn receive(&mut self) -> Option<ClientMessage<ID>> {
        self.messages.pop()
    }
}
