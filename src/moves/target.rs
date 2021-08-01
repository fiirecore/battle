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
    Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum MoveTargetLocation {
    Opponent(usize), // maybe add TrainerId
    Team(usize),
    User,
}

impl MoveTargetLocation {
    pub fn user() -> Vec<Self> {
        vec![Self::User]
    }

    pub fn opponent(index: usize) -> Vec<Self> {
        vec![Self::Opponent(index)]
    }

    pub fn team(index: usize) -> Vec<Self> {
        vec![Self::Team(index)]
    }

    pub fn allies(user: usize, len: usize) -> Vec<Self> {
        let mut vec = Vec::with_capacity(len - 1);
        for i in 0..len {
            if i != user {
                vec.push(Self::Team(i));
            }
        }
        vec
    }

    pub fn opponents(size: usize) -> Vec<Self> {
        (0..size).into_iter().map(Self::Opponent).collect()
    }

    pub fn user_and_allies(user: usize, player: usize) -> Vec<Self> {
        let mut vec = Vec::with_capacity(player);
        for i in 0..player {
            if i != user {
                vec.push(Self::Team(i));
            } else {
                vec.push(Self::User);
            }
        }
        vec
    }

    pub fn all_pokemon(user: usize, player: usize, opponent: usize) -> Vec<Self> {
        let mut vec = Vec::with_capacity(player + opponent);
        for i in 0..player {
            if i != user {
                vec.push(Self::Team(i));
            } else {
                vec.push(Self::User);
            }
        }
        vec.extend((0..opponent).into_iter().map(Self::Opponent));
        vec
    }

    pub fn all_other_pokemon(user: usize, player: usize, opponent: usize) -> Vec<Self> {
        let mut vec = Vec::with_capacity(player + opponent - 1);
        for i in 0..player {
            if i != user {
                vec.push(Self::Team(i));
            }
        }
        vec.extend((0..opponent).into_iter().map(Self::Opponent));
        vec
    }
}