use crate::{moves::BattleMove, party::PartyIndex};

#[derive(Debug, Clone)]
pub struct ActivePokemon {
    pub index: usize,
    pub queued_move: Option<BattleMove>,
}

impl ActivePokemon {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            queued_move: None,
        }
    }

    pub fn use_move(&mut self) -> Option<BattleMove> {
        self.queued_move.take()
    }
}

impl PartyIndex for ActivePokemon {
    fn index(&self) -> usize {
        self.index
    }
}