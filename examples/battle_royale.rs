use std::ops::{Deref, Range};

use firecore_pokedex::{
    item::Item,
    moves::{owned::SavedMove, Move, MoveCategory, MoveTarget},
    pokemon::{
        data::{Breeding, LearnableMove, Training},
        owned::SavedPokemon,
        party::Party,
        stat::StatSet,
        Pokemon,
    },
    types::{PokemonType, Types},
    BasicDex,
};

use firecore_battle::{
    default_engine::moves::{MoveExecution, MoveUse},
    moves::damage::DamageKind,
    prelude::*,
};

const POKEMON: Range<u16> = 0..6;

#[derive(Clone)]
struct Container<T>(T);

impl<T> From<T> for Container<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T> Deref for Container<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        .init()
        .unwrap();

    let mut pokedex = BasicDex::<Pokemon, Container<_>>::default();
    let mut movedex = BasicDex::<Move, Container<_>>::default();
    let itemdex = BasicDex::<Item, Container<_>>::default();

    let move_id = [
        "default".parse().unwrap(),
        "script".parse().unwrap(),
        "damage".parse().unwrap(),
    ];

    movedex.insert(Move {
        id: move_id[0],
        name: "Test Move".to_owned(),
        category: MoveCategory::Physical,
        pokemon_type: PokemonType::Normal,
        accuracy: Some(50),
        power: Some(50),
        pp: 50,
        priority: 0,
        target: MoveTarget::Opponent,
        contact: false,
        crit_rate: 0,
    });

    movedex.insert(Move {
        id: move_id[1],
        name: "Script Move".to_owned(),
        category: MoveCategory::Physical,
        pokemon_type: PokemonType::Normal,
        accuracy: Some(50),
        power: None,
        pp: 50,
        priority: 0,
        target: MoveTarget::Opponent,
        contact: false,
        crit_rate: 0,
    });

    movedex.insert(Move {
        id: move_id[2],
        name: "Damage".to_owned(),
        category: MoveCategory::Physical,
        pokemon_type: PokemonType::Fighting,
        accuracy: Some(80),
        power: Some(90),
        pp: 50,
        priority: 0,
        target: MoveTarget::Opponent,
        contact: false,
        crit_rate: 2,
    });

    for id in POKEMON {
        pokedex.insert(Pokemon {
            id,
            name: format!("Test {}", id),
            types: Types {
                primary: PokemonType::Normal,
                secondary: Some(PokemonType::Ice),
            },
            moves: vec![LearnableMove(0, move_id[0]), LearnableMove(0, move_id[1])],
            base: StatSet::uniform(70),
            species: "Test".to_owned(),
            height: 100,
            weight: 100,
            evolution: None,
            training: Training {
                base_exp: 80,
                growth: Default::default(),
            },
            breeding: Breeding { gender: None },
        });
    }

    type RngType = rand::rngs::SmallRng;
    use rand::prelude::SeedableRng;

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(325218);

    let mut random = RngType::seed_from_u64(seed);

    let party: Party<_> = POKEMON
        .into_iter()
        .enumerate()
        .map(|(index, id)| SavedPokemon {
            pokemon: id,
            level: 10 + (index as u8) * 20,
            ..Default::default()
        })
        .map(|mut o| {
            for m in move_id {
                o.moves.push(SavedMove::from(m));
            }
            o
        })
        .collect();

    const AS: usize = 2;

    let mut players: Vec<_> = (1..100)
        .into_iter()
        .map(|_| BattleAi::<u8, (), RngType, _, _, _>::new(random.clone()))
        .collect();

    let mut battle = Battle::new(
        BattleData::default(),
        &mut random,
        AS,
        &pokedex,
        &movedex,
        &itemdex,
        players.iter().enumerate().map(|(id, player)| PlayerData {
            id: id as _,
            name: Some(format!("Player {}", id)),
            party: party.clone(),
            bag: Default::default(),
            trainer: Some(()),
            settings: PlayerSettings { gains_exp: false },
            endpoint: Box::new(player.endpoint().clone()),
        }),
    );

    let mut engine = DefaultEngine::new::<u8, RngType>();

    engine.moves.insert(
        move_id[0],
        MoveExecution::Actions(vec![MoveUse::Damage(DamageKind::Power(50))]),
    );

    engine.moves.insert(
        move_id[2],
        MoveExecution::Actions(vec![MoveUse::Damage(DamageKind::Power(90))]),
    );

    engine.moves.insert(move_id[1], MoveExecution::Script);

    let script = "
    let results = [];

    for target in targets {
        switch user.throw_move(random, move) {
            false => results.push(Miss(user)),
            true => {
                let result = damage(target.hp);
                results.push(Damage(target, result));
            }
        }
    }

    results
    ";

    engine.scripting.moves.insert(move_id[1], script.to_owned());

    while !battle.finished() {
        battle.update(&mut random, &mut engine, &movedex, &itemdex);
        for player in players.iter_mut() {
            player.update(&pokedex, &movedex, &itemdex);
        }
    }

    log::info!(
        "{} wins!",
        match battle.winner() {
            Some(id) => format!("Player #{}", id),
            None => "No one".to_owned(),
        }
    );
}
