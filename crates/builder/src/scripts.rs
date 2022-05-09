use hashbrown::HashMap;
use std::{
    fs::{read_dir, read_to_string},
    path::Path,
};

use battle::pokedex::moves::MoveId;

pub type Scripts = HashMap<MoveId, String>;

pub fn get_moves<P: AsRef<Path>>(scripts: P) -> Scripts {
    let scripts = scripts.as_ref();

    read_dir(scripts)
        .unwrap_or_else(|err| {
            panic!(
                "Could not read move scripts directory at {:?} with error {}",
                scripts, err
            )
        })
        .flatten()
        .map(|d| d.path())
        .filter(|p| p.is_file())
        .map(|path| {
            (
                path.file_stem()
                    .unwrap_or_else(|| {
                        panic!("Could not get file name for script file at path {:?}", path,)
                    })
                    .to_string_lossy()
                    .parse::<MoveId>()
                    .unwrap_or_else(|err| {
                        panic!(
                            "Could not parse move script file {:?} into MoveScriptId with error {}",
                            path, err
                        )
                    }),
                read_to_string(&path).unwrap_or_else(|err| {
                    panic!(
                        "Could not read move file at {:?} to string with error {}",
                        path, err
                    )
                }),
            )
        })
        .collect()
}
