pub extern crate firecore_battle as battle;

use std::path::Path;

pub mod moves;
pub mod scripts;

pub fn compile<P: AsRef<Path>>(moves: P, scripts: P) -> (moves::Moves, scripts::Scripts) {
    (moves::get_moves(moves), scripts::get_moves(scripts))
}
