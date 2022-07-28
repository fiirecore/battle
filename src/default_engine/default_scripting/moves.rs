pub use pokedex::{moves::MoveCategory, types::PokemonType};

mod ailment;
mod damage;
mod moves;
mod pokemon;
mod random;
mod result;

pub use ailment::*;
pub use damage::*;
pub use moves::*;
pub use pokemon::*;
pub use random::*;
pub use result::*;

// pub fn option_some<T>(t: T) -> Option<T> {
//     Option::Some(t)
// }

// pub fn option_none<T>() -> Option<T> {
//     Option::None
// }
