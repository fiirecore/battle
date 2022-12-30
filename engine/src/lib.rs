extern crate firecore_battle as battle;

use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::Hash,
    marker::PhantomData,
};

use hashbrown::HashMap;
use rand::Rng;

use battle::{
    data::BattleData,
    engine::*,
    moves::{BattleMove, DamageResult, ClientDamage},
    pokedex::{item::ItemId, moves::MoveId, pokemon::Health},
    pokemon::{ActivePosition, Indexed, TeamIndex, BattlePokemon, throw_move},
    select::*,
};

use crate::scripting::ScriptingEngine;

pub mod item;
mod target;
use self::item::*;

pub mod moves;
use self::moves::*;

pub mod scripting;

pub type EngineItems = HashMap<ItemId, BattleItemExecution>;
pub type EngineMoves = HashMap<MoveId, EngineMove>;

pub type DefaultEngine<ID, T> = DefaultBattleEngine<ID, T, scripting::RhaiScriptingEngine>;

pub struct DefaultBattleEngine<
    ID: Clone + Hash + Eq + Send + Sync + 'static,
    T: Send + Sync + 'static,
    S: ScriptingEngine<ID, T> + Send + Sync,
> {
    pub items: EngineItems,
    pub moves: EngineMoves,
    pub scripting: S,
    _p: PhantomData<(ID, T)>,
}

impl<
        ID: Clone + Hash + Eq + Send + Sync + 'static,
        T: Send + Sync + 'static,
        S: ScriptingEngine<ID, T> + Send + Sync + 'static,
    > DefaultBattleEngine<ID, T, S>
{
    pub fn with_scripting_engine(scripting: S) -> Self {
        Self {
            items: Default::default(),
            moves: Default::default(),
            scripting,
            _p: Default::default(),
        }
    }
}


pub struct DefaultEngineData<
ID: Clone + Hash + Eq + Send + Sync + 'static,
T: Send + Sync + 'static,
S: ScriptingEngine<ID, T> + Send + Sync> {
    scripting: S::Data,
    _p: PhantomData<(ID, T)>,
}

impl<ID: Clone + Hash + Eq + Send + Sync + 'static, T: Send + Sync + 'static>
    DefaultBattleEngine<ID, T, scripting::RhaiScriptingEngine>
{
    pub fn new<R: Rng + Clone + Send + Sync + 'static>() -> Self {
        Self {
            items: Default::default(),
            moves: Default::default(),
            scripting: scripting::RhaiScriptingEngine::new::<ID, R>(),
            _p: Default::default(),
        }
    }
}

impl<
ID: Clone + Hash + Eq + Send + Sync + 'static,
T: Send + Sync + 'static,
S: ScriptingEngine<ID, T> + Send + Sync> Default for DefaultEngineData<ID, T, S> {
    fn default() -> Self {
        Self { scripting: Default::default(), _p: Default::default() }
    }
}

impl<
        ID: Clone + Hash + Eq + Send + Sync + 'static,
        T: Send + Sync + 'static,
        S: ScriptingEngine<ID, T> + Send + Sync + 'static,
    > BattleEngine<ID, T> for DefaultBattleEngine<ID, T, S>
{
    type ExecutionError = DefaultError<S::ExecutionError>;

    /// Current battle data
    type Data = DefaultEngineData<ID, T, S>;

    fn select(
        &self,
        data: &mut Self::Data,
        active: ActivePosition,
        selection: &BattleSelection<ID>,
        player: &mut BattlePlayer<ID, T>,
    ) -> SelectMessage {
        let mut confirm = SelectConfirm::Other;

        match selection {
            BattleSelection::Move(id, ..) => {
                if let Some(pokemon) = player.party.active(active) {
                    if let Some(m) = pokemon.moves.iter().find(|m| m.id() == id) {
                        if m.is_empty() {
                            return SelectMessage::Request(Some(SelectReason::NoPP));
                        } else {
                            confirm = SelectConfirm::Move(id.clone(), 1);
                        }
                    } else {
                        return SelectMessage::Request(Some(SelectReason::MissingAction));
                    }
                }
            }
            BattleSelection::Pokemon(index) => unreachable!(),
            BattleSelection::Item(id) => {
                if !player.bag.contains(&id.1) {
                    return SelectMessage::Request(Some(SelectReason::MissingAction));
                }
            }
        }

        SelectMessage::Confirm(confirm)
    }

    fn execute(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        action: ExecuteAction<ID>,
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError> {
        match action {
            ExecuteAction::Move(id, user, targeting) => match self.moves.get(id) {
                Some(m) => {
                    let targets = target::create_targets(&players, user, &m.data, targeting, random);

                    match &m.usage {
                        MoveExecution::Actions(actions) => {
                            let mut results = Vec::<Indexed<ID, PublicAction>>::new();
                            for target_id in targets {
                                match players.get(target_id.team())
                                    .and_then(|p| p.party.active(target_id.index()))
                                {
                                    Some(target) => match throw_move(random, m.data.accuracy) {
                                        true => {
                                            let mut a = Vec::new();
                                            move_usage(
                                                &Indexed(
                                                    user.clone(),
                                                    players.get(user.team())
                                                        .and_then(|p| p.party.active(user.index()))
                                                        .unwrap(),
                                                ),
                                                random,
                                                &mut a,
                                                &actions,
                                                &m.data,
                                                Indexed(target_id, target),
                                            );
                                            for action in a {
                                                run_action(action, battle, user, &mut results, players);
                                            }
                                        }
                                        false => {
                                            results.push(Indexed(user.clone(), PublicAction::Miss));
                                        }
                                    },
                                    None => unreachable!(),
                                }
                            }
                            Ok(results)
                        }
                        MoveExecution::Script => {
                            return self
                                .scripting
                                .execute_move(&mut data.scripting, random, battle, &m.data, user, targets, players)
                                .map_err(DefaultError::Script);
                        }
                        MoveExecution::None => Err(DefaultError::Unimplemented),
                    }
                }
                None => Err(DefaultError::Unknown),
            },
            ExecuteAction::Item(id, user, target) => match self.items.get(id) {
                Some(execution) => match execution {
                    BattleItemExecution::Normal(..) => {
                        // to - do: fix this function
                        // log::debug!("fix OwnedPokemon::try_use_item");
                        // match players.get_mut(&target) {
                        //     Some(pokemon) => {
                        //         pokemon.try_use_item(item);
                        //         Ok(vec![])
                        //     }
                        //     None => Err(ItemError::NoTarget),
                        // }
                        Err(DefaultError::Unimplemented)
                    }
                    BattleItemExecution::Script => {
                        return self
                            .scripting
                            .execute_item(&mut data.scripting, random, battle, id, user, target, players)
                            .map_err(DefaultError::Script);
                    }
                    BattleItemExecution::Pokeball => match battle.versus.is_wild() {
                        true => Ok(match players.iter().find(|p| p.id() == target.team()).and_then(|p| p.party.active(target.index())).is_some() {
                            true => {
                                todo!("add the ability for the battle to send results to a single client");
                                //     vec![
                                //     Indexed(target.clone(), PublicAction::Reveal),
                                //     ActionResult::Remove("catch".parse().unwrap()),
                                // ]
                            },
                            false => Vec::new(),
                        }),
                        false => todo!(),
                    },
                },
                None => Err(DefaultError::Unknown),
            },
        }
    }

    fn post(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, PublicAction>>, Self::ExecutionError> {
        Ok(vec![])
    }

    fn reset(&self, data: &mut Self::Data) {

    }

    fn get_move(&self, id: &MoveId) -> Option<&BattleMove> {
        self.moves.get(id).map(|m| &m.data)
    }
    
}

#[derive(Debug)]
pub enum DefaultError<S: Error = NoScriptError> {
    Script(S),
    Unknown,
    NoTarget,
    NoExecution,
    Unimplemented,
}

impl<S: Error> Error for DefaultError<S> {}

impl<S: Error> Display for DefaultError<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Script(err) => Display::fmt(err, f),
            other => Debug::fmt(other, f),
        }
    }
}

#[derive(Debug)]
pub enum NoScriptError {
    NoScriptEngine,
}

impl Error for NoScriptError {}

impl Display for NoScriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "No scripting engine!")
    }
}

fn run_action<ID: PartialEq + Clone, T>(
    Indexed(target_id, action): Indexed<ID, ActionResult>,
    data: &BattleData,
    user_id: &TeamIndex<ID>,
    actions: &mut Vec<Indexed<ID, PublicAction>>,
    players: &mut PlayerQuery<ID, T>
) {
    match players.get_mut(target_id.team())
    {
        Some(player) => {
            match player.party.active_mut(target_id.index()) {
                Some(target) => {
                    /// calculates hp and adds it to actions
                    fn on_damage<ID>(
                        location: TeamIndex<ID>,
                        pokemon: &mut BattlePokemon,
                        actions: &mut Vec<Indexed<ID, PublicAction>>,
                        result: DamageResult<Health>,
                    ) {
                        pokemon.hp = pokemon.hp.saturating_sub(result.damage);
                        actions.push(Indexed(
                            location,
                            PublicAction::SetHP(ClientDamage::Result(DamageResult {
                                damage: pokemon.percent_hp(),
                                effective: result.effective,
                                crit: result.crit,
                            })),
                        ));
                    }

                    let t_id = target_id.clone();

                    match action {
                        ActionResult::Damage(result) => {
                            on_damage(target_id, target, actions, result)
                        }
                        ActionResult::Ailment(ailment) => {
                            target.ailment = ailment;
                            actions.push(Indexed(target_id, PublicAction::Ailment(ailment)));
                        }
                        ActionResult::Heal(health) => {
                            let hp = health.unsigned_abs();
                            target.hp = match health.is_positive() {
                                true => target.hp + hp.min(target.max_hp()),
                                false => target.hp.saturating_sub(hp),
                            };
                            actions.push(Indexed(
                                target_id,
                                PublicAction::SetHP(ClientDamage::Number(target.percent_hp())),
                            ));
                        }
                        ActionResult::Stat(stat, stage) => {
                            target.stages.change_stage(stat, stage);
                            actions
                                .push(Indexed(target_id, PublicAction::AddStat(stat, stage)));
                        }
                        ActionResult::Cancel(reason) => {
                            actions.push(Indexed(target_id, PublicAction::Cancel(reason)))
                        }
                        ActionResult::Miss => {
                            actions.push(Indexed(target_id, PublicAction::Miss));
                        }
                        ActionResult::Reveal(full) => {
                            todo!();
                            // player.party.know(index)
                        }
                        ActionResult::Remove(reason) => {
                            todo!();
                            // player.party.remove_active(active);
                        }
                        ActionResult::Fail => todo!(),
                    }

                    if target.fainted() {

                        let experience = target.battle_exp_from(data.versus);

                        player.party.remove_active(t_id.index());

                        if &player.party.id != t_id.team() {
                            drop(player);
                            let user = players
                                .iter_mut()
                                .find(|p| &p.party.id == user_id.team())
                                .unwrap();

                            if user.settings.gains_exp {
                                let user = user.party.active_mut(user_id.index()).unwrap();

                                let levels = user.p.add_exp(experience);

                                let moves = user.p.pokemon.moves_at(levels).copied().collect::<Vec<_>>();

                                user.try_learn_moves(moves);

                                println!("todo push exp action");

                                // actions.push(Indexed(
                                //     user_id.clone(),
                                //     PublicAction::SetExp(experience, user.level),
                                // ));
                            }
                        }
                    }
                }
                None => unreachable!(),
            }
        }
        None => unreachable!(),
    }
}
