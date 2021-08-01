use crate::{moves::BattleMove, party::PartyIndex};

#[derive(Debug, Clone, Copy)]
pub struct ActivePokemon {
    pub index: usize,
    pub queued_move: Option<BattleMove>,
}

impl PartyIndex for ActivePokemon {
    fn index(&self) -> usize {
        self.index
    }
}

impl From<usize> for ActivePokemon {
    fn from(index: usize) -> Self {
        Self {
            index,
            queued_move: None,
        }
    }
}