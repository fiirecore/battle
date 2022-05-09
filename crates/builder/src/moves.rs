use hashbrown::HashMap;
use std::{
    fs::{read_dir, read_to_string},
    path::Path,
};

use battle::{default_engine::moves::MoveExecution, pokedex::moves::MoveId};

pub type Moves = HashMap<MoveId, MoveExecution>;

pub fn get_moves<P: AsRef<Path>>(moves: P) -> Moves {
    let moves = moves.as_ref();

    read_dir(moves)
        .unwrap_or_else(|err| {
            panic!(
                "Could not read moves directory at {:?} with error {}",
                moves, err
            )
        })
        .flat_map(|entry| match entry.map(|entry| entry.path()) {
            Ok(path) => match path.is_file() {
                true => {
                    let (id, usage) = ron::from_str::<(MoveId, MoveExecution)>(
                        &read_to_string(&path).unwrap_or_else(|err| {
                            panic!(
                                "Could not read move file at {:?} to string with error {}",
                                path, err
                            )
                        }),
                    )
                    .unwrap_or_else(|err| {
                        panic!("Could not parse move file at {:?} with error {}", path, err)
                    });
                    Some((id, usage))
                }
                false => None,
            },
            Err(err) => {
                eprintln!("Could not read directory entry with error {}", err);
                None
            }
        })
        .collect()
}
