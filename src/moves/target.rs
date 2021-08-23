use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum MoveTargetInstance {
    Any(bool, usize),
    Ally(usize),
    Allies,
    UserAndAllies,
    UserOrAlly(usize),
    User,
    Opponent(usize),
    AllOpponents,
    RandomOpponent,
    AllOtherPokemon,
    AllPokemon,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum TargetLocation {
    Opponent(usize), // maybe add TrainerId
    Team(usize),
    User,
}

use core::iter::once;

impl TargetLocation {

    pub fn is_team(&self) -> bool {
        match self {
            Self::Opponent(..) => false,
            _ => true,
        }
    }

    pub fn user() -> impl Iterator<Item = Self> {
        once(Self::User)
    }

    pub fn opponent(index: usize) -> impl Iterator<Item = Self> {
        once(Self::Opponent(index))
    }

    pub fn team(index: usize) -> impl Iterator<Item = Self> {
        once(Self::Team(index))
    }

    pub fn allies(user: usize, size: usize) -> impl Iterator<Item = Self> {
        (0..size).into_iter().filter(move |index| index != &user).map(Self::Team)
    }

    pub fn opponents(size: usize) -> impl Iterator<Item = Self> {
        (0..size).into_iter().map(Self::Opponent)
    }

    pub fn user_and_allies(user: usize, size: usize) -> impl Iterator<Item = Self> {
        Self::allies(user, size).chain(once(Self::User))
    }

    pub fn all_pokemon(user: usize, player_size: usize, opponent_size: usize) -> impl Iterator<Item = Self> {
        Self::opponents(opponent_size).chain(Self::user_and_allies(user, player_size))
    }

    pub fn all_other_pokemon(user: usize, player_size: usize, opponent_size: usize) -> impl Iterator<Item = Self> {
        Self::opponents(opponent_size).chain(Self::allies(user, player_size))
    }
}