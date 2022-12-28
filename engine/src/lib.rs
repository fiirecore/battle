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
    host::BattlePlayer,
    moves::BattleMove,
    pokedex::{item::ItemId, moves::MoveId},
    pokemon::{ActivePosition, Indexed, TeamIndex},
    select::*,
};

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

pub trait ScriptingEngine<ID, T> {
    /// Current battle data
    type Data;

    type ExecutionError: Error;

    fn execute_move<'a, 'b: 'a>(
        &'b self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        m: &BattleMove,
        user: Indexed<ID, &mut BattlePokemon>,
        targets: Vec<TeamIndex<ID>>,
        players: &'a mut PlayerQuery<'a, ID, T>,
    ) -> Result<Vec<Indexed<ID, ClientMoveAction>>, Self::ExecutionError>;

    fn execute_item(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        item: &ItemId,
        user: &ID,
        target: TeamIndex<ID>,
        players: &mut PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, ClientMoveAction>>, Self::ExecutionError>;
}

impl<
        ID: Clone + Hash + Eq + Send + Sync + 'static,
        T: Send + Sync + 'static,
        S: ScriptingEngine<ID, T> + Send + Sync + 'static,
    > BattleEngine<ID, T> for DefaultBattleEngine<ID, T, S>
{
    type ExecutionError = DefaultError<S::ExecutionError>;

    /// Current battle data
    type Data = ();

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
                        if m.pp() == 0 {
                            return SelectMessage::Request(Some(SelectReason::NoPP));
                        } else {
                            confirm = SelectConfirm::Move(id.clone(), 1);
                        }
                    } else {
                        return SelectMessage::Request(Some(SelectReason::MissingAction));
                    }
                }
            }
            BattleSelection::Pokemon(index) => {
                if player.party.active_contains(*index) || *index >= player.party.pokemon.len() {
                    return SelectMessage::Request(Some(SelectReason::MissingAction));
                }
            }
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
        players: PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, ClientMoveAction>>, Self::ExecutionError> {
        todo!();
        // match action {
        //     ExecuteAction::Move(id, user, targeting) => match self.moves.get(id) {
        //         Some(m) => {
        //             let targets = target::create_targets(players, user, &m.data, targeting, random);

        //             match &m.usage {
        //                 MoveExecution::Actions(actions) => {
        //                     let mut results = Vec::new();
        //                     for target_id in targets {
        //                         match players
        //                             .iter()
        //                             .find(|p| p.id() == target_id.team())
        //                             .and_then(|p| p.party.active(target_id.index()))
        //                         {
        //                             Some(target) => match throw_move(random, m.data.accuracy) {
        //                                 true => {
        //                                     move_usage(
        //                                         Indexed(
        //                                             user.clone(),
        //                                             players
        //                                                 .iter()
        //                                                 .find(|p| p.id() == user.team())
        //                                                 .and_then(|p| p.party.active(user.index()))
        //                                                 .unwrap(),
        //                                         ),
        //                                         random,
        //                                         &mut results,
        //                                         &actions,
        //                                         &m.data,
        //                                         Indexed(target_id, target),
        //                                     );
        //                                 }
        //                                 false => {
        //                                     results.push(Indexed(user.clone(), ClientMoveAction::Miss))
        //                                 }
        //                             },
        //                             None => unreachable!(),
        //                         }
        //                     }
        //                     Ok(results)
        //                 }
        //                 MoveExecution::Script => {
        //                     return self
        //                         .scripting
        //                         .execute_move(data, random, battle, &m.data, user, targets, players)
        //                         .map_err(DefaultError::Script);
        //                 }
        //                 MoveExecution::None => Err(DefaultError::Unimplemented),
        //             }
        //         }
        //         None => Err(DefaultError::Unknown),
        //     },
        //     ExecuteAction::Item(id, user, target) => match self.items.get(id) {
        //         Some(execution) => match execution {
        //             BattleItemExecution::Normal(..) => {
        //                 // to - do: fix this function
        //                 // log::debug!("fix OwnedPokemon::try_use_item");
        //                 // match players.get_mut(&target) {
        //                 //     Some(pokemon) => {
        //                 //         pokemon.try_use_item(item);
        //                 //         Ok(vec![])
        //                 //     }
        //                 //     None => Err(ItemError::NoTarget),
        //                 // }
        //                 Err(DefaultError::Unimplemented)
        //             }
        //             BattleItemExecution::Script => {
        //                 return self
        //                     .scripting
        //                     .execute_item(data, random, battle, id, user, target, players)
        //                     .map_err(DefaultError::Script);
        //             }
        //             BattleItemExecution::Pokeball => match battle.versus.is_wild() {
        //                 true => Ok(match players.iter().find(|p| p.id() == target.team()).and_then(|p| p.party.active(target.index())).is_some() {
        //                     true => vec![
        //                         ActionResult::Reveal(true),
        //                         ActionResult::Remove("catch".parse().unwrap()),
        //                     ],
        //                     false => Vec::new(),
        //                 }),
        //                 false => todo!(),
        //             },
        //         },
        //         None => Err(DefaultError::Unknown),
        //     },
        // }
    }

    fn get_move(&self, id: &MoveId) -> Option<&BattleMove> {
        self.moves.get(id).map(|m| &m.data)
    }

    fn post(
        &self,
        data: &mut Self::Data,
        random: &mut (impl Rng + Clone + Send + Sync + 'static),
        battle: &mut BattleData,
        players: PlayerQuery<ID, T>,
    ) -> Result<Vec<Indexed<ID, ClientMoveAction>>, Self::ExecutionError> {
        todo!()
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

// fn run_action<ID: PartialEq + Clone, T>(
//     Indexed(target_id, action): Indexed<ID, ActionResult>,
//     user_id: &TeamIndex<ID>,
//     movedex: &firecore_pokedex::Dex<firecore_pokedex::moves::Move>,
//     actions: &mut Vec<Indexed<ID, ClientMoveAction>>,
//     players: &mut PlayerQuery<ID, T>
// ) -> Vec<Indexed<ID, ClientMoveAction>> {
//     match players
//         .iter_mut()
//         .find(|p| &p.party.id == target_id.team())
//     {
//         Some(mut player) => {
//             match player.party.active_mut(target_id.index()) {
//                 Some(target) => {
//                     /// calculates hp and adds it to actions
//                     fn on_damage<ID>(
//                         location: TeamIndex<ID>,
//                         pokemon: &mut crate::engine::BattlePokemon,
//                         actions: &mut Vec<Indexed<ID, ClientMoveAction>>,
//                         result: DamageResult<firecore_pokedex::pokemon::Health>,
//                     ) {
//                         pokemon.hp = pokemon.hp.saturating_sub(result.damage);
//                         actions.push(Indexed(
//                             location,
//                             ClientMoveAction::SetHP(ClientDamage::Result(DamageResult {
//                                 damage: pokemon.percent_hp(),
//                                 effective: result.effective,
//                                 crit: result.crit,
//                             })),
//                         ));
//                     }

//                     let t_id = target_id.clone();

//                     match action {
//                         ActionResult::Damage(result) => {
//                             on_damage(target_id, target, &mut actions, result)
//                         }
//                         ActionResult::Ailment(ailment) => {
//                             target.ailment = ailment;
//                             actions.push(Indexed(target_id, ClientMoveAction::Ailment(ailment)));
//                         }
//                         ActionResult::Heal(health) => {
//                             let hp = health.unsigned_abs();
//                             target.hp = match health.is_positive() {
//                                 true => target.hp + hp.min(target.max_hp()),
//                                 false => target.hp.saturating_sub(hp),
//                             };
//                             actions.push(Indexed(
//                                 target_id,
//                                 ClientMoveAction::SetHP(ClientDamage::Number(target.percent_hp())),
//                             ));
//                         }
//                         ActionResult::Stat(stat, stage) => {
//                             target.stages.change_stage(stat, stage);
//                             actions
//                                 .push(Indexed(target_id, ClientMoveAction::AddStat(stat, stage)));
//                         }
//                         ActionResult::Cancel(reason) => {
//                             actions.push(Indexed(target_id, ClientMoveAction::Cancel(reason)))
//                         }
//                         ActionResult::Miss => {
//                             actions.push(Indexed(target_id, ClientMoveAction::Miss));
//                         }
//                         ActionResult::Reveal(full) => {
//                             todo!();
//                             // player.party.know(index)
//                         }
//                         ActionResult::Remove(reason) => {
//                             todo!();
//                             // player.party.remove_active(active);
//                         }
//                         ActionResult::Fail => todo!(),
//                     }

//                     if target.fainted() {
//                         let experience = target.battle_exp_from(data.versus);

//                         player.party.remove_active(t_id.index());

//                         if &player.party.id != t_id.team() {
//                             let mut user = players
//                                 .iter_mut()
//                                 .find(|p| &p.party.id == user_id.team())
//                                 .unwrap();

//                             if user.settings.gains_exp {
//                                 let user = user.party.active_mut(user_id.index()).unwrap();

//                                 user.try_learn_moves(user.p.add_exp(movedex, experience).copied());

//                                 actions.push(Indexed(
//                                     user_id.clone(),
//                                     ClientMoveAction::SetExp(experience, user.level),
//                                 ));
//                             }
//                         }
//                     }
//                 }
//                 None => unreachable!(),
//             }
//         }
//         None => unreachable!(),
//     }
// }
