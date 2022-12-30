use std::{ops::{Deref, Range}, sync::Arc};

use firecore_battle_engine::{
    moves::{EngineMove, MoveExecution, MoveUse},
    DefaultEngine,
};
use firecore_pokedex::{
    item::Item,
    moves::{owned::SavedMove, Move},
    pokemon::{
        data::{Breeding, LearnableMove, Training},
        owned::SavedPokemon,
        party::Party,
        stat::StatSet,
        Level, Pokemon, PokemonId,
    },
    types::{PokemonType, PokemonTypes},
    Dex,
};

use firecore_battle::{
    ai::BattleAi,
    data::BattleData,
    host::{Battle, PlayerData},
    moves::{BattleMove, Contact, DamageKind, MoveCategory, MoveTarget},
    player::PlayerSettings,
};
use rand::seq::IteratorRandom;

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

type Id = u8;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        .init()
        .unwrap();

    let mut pokedex = Dex::<Pokemon>::default();
    let mut movedex = Dex::<Move>::default();
    let itemdex = Dex::<Item>::default();

    let move_id = [
        "default".parse().unwrap(),
        "script1".parse().unwrap(),
        "damage".parse().unwrap(),
        "script2".parse().unwrap(),
    ];

    movedex.insert(Move {
        id: move_id[0],
        name: "Test Move".to_owned(),
        pp: 50,
    });

    movedex.insert(Move {
        id: move_id[1],
        name: "Script Move".to_owned(),
        pp: 50,
    });

    movedex.insert(Move {
        id: move_id[2],
        name: "Damage".to_owned(),
        pp: 50,
    });

    movedex.insert(Move {
        id: move_id[3],
        name: "Aromatherapy".to_owned(),
        pp: 50,
    });

    for id in POKEMON {
        pokedex.insert(Pokemon {
            id: PokemonId(id),
            name: format!("Test {}", id),
            types: PokemonTypes {
                primary: PokemonType::Normal,
                secondary: Some(PokemonType::Ice),
            },
            moves: vec![
                LearnableMove(0, move_id[0]),
                LearnableMove(0, move_id[1]),
                LearnableMove(0, move_id[2]),
                LearnableMove(0, move_id[3]),
            ],
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
            pokemon: PokemonId(id),
            level: 10 + (index as Level) * 20,
            ..Default::default()
        })
        .map(|mut o| {
            for m in move_id {
                o.moves.push(SavedMove::from(m));
            }
            o
        })
        .flat_map(|p| p.init(&mut random, &pokedex, &movedex, &itemdex))
        .collect();

    const AS: usize = 2;

    let mut players: Vec<_> = (1..100)
        .into_iter()
        .map(|_| BattleAi::new())
        .collect();

    let mut battle = Battle::<Id, (), DefaultEngine<_, _>>::new(
        BattleData {
            active: AS,
            ..Default::default()
        },
        players.iter().enumerate().map(|(id, player)| PlayerData {
            id: id as _,
            name: Some(format!("Player {}", id)),
            party: party.clone(),
            bag: Default::default(),
            trainer: Some(()),
            settings: PlayerSettings { gains_exp: false },
            endpoint: Arc::new(player.endpoint().clone()),
        }),
    );

    let mut engine = DefaultEngine::<Id, ()>::new::<RngType>();

    engine.moves.insert(
        move_id[0],
        EngineMove {
            data: BattleMove {
                id: move_id[0],
                category: MoveCategory::Physical,
                pokemon_type: PokemonType::Normal,
                accuracy: Some(50),
                power: Some(50),
                priority: 0,
                target: MoveTarget::Opponent,
                contact: Contact::default(),
                crit_rate: 0,
            },
            usage: MoveExecution::Actions(vec![MoveUse::Damage(DamageKind::Power(50))]),
        },
    );

    engine.moves.insert(
        move_id[2],
        EngineMove {
            data: BattleMove {
                id: move_id[2],
                category: MoveCategory::Physical,
                pokemon_type: PokemonType::Fighting,
                accuracy: Some(80),
                power: Some(90),
                priority: 0,
                target: MoveTarget::Opponent,
                contact: Contact(true),
                crit_rate: 2,
            },
            usage: MoveExecution::Actions(vec![MoveUse::Damage(DamageKind::Power(90))]),
        },
    );

    engine.moves.insert(
        move_id[1],
        EngineMove {
            data: BattleMove {
                id: move_id[1],
                category: MoveCategory::Physical,
                pokemon_type: PokemonType::Normal,
                accuracy: Some(50),
                power: None,
                priority: 0,
                target: MoveTarget::Opponent,
                contact: Contact(false),
                crit_rate: 0,
            },
            usage: MoveExecution::Script,
        },
    );

    engine.moves.insert(
        move_id[3],
        EngineMove {
            data: BattleMove {
                id: move_id[3],
                category: MoveCategory::Status,
                pokemon_type: PokemonType::Grass,
                accuracy: Some(80),
                power: None,
                priority: 0,
                target: MoveTarget::UserAndAllies,
                contact: Contact(false),
                crit_rate: 0,
            },
            usage: MoveExecution::Script,
        },
    );

    let script1 = r#"

    fn use_move(move, user, targets) {
        let results = [];

        for target in targets {
            switch user.throw_move(random, move) {
                false => {
                    results.push(Miss(user));
                },
                true => {
                    let result = damage(target.hp);
                    results.push(Damage(target, result));
                }
            }
        }
    
        results
    }

    "#;

    let script2 = r#"

    fn use_move(move, user, targets) {
        let results = [];

        switch user.throw_move(random, move) {
            false => {
                results.push(Miss(user));
            },
            true => {
                for target in targets {
                    results.push(Ailment(target, CLEAR));
                }
            }
        }
    
        results
    }

    "#;

    engine
        .scripting
        .moves
        .insert(move_id[1], script1.to_owned());
    engine
        .scripting
        .moves
        .insert(move_id[3], script2.to_owned());

    while battle.running() {
        battle.update(&mut random, &mut engine, &movedex).unwrap();
        for player in players.iter_mut() {
            player.update(&mut random, &pokedex, &movedex, &itemdex).unwrap();
        }
        println!("{}", players.get(firecore_battle::host::test.load(std::sync::atomic::Ordering::Relaxed) as usize).unwrap());
        // if let Some(player) = players.iter_mut().filter(|p| p.active()).choose(&mut random) {
        //     log::info!("Random AI Info: {player}");
        // }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    log::info!(
        "{} wins!",
        match battle.winner().flatten() {
            Some(id) => format!("Player #{}", id),
            None => "No one".to_owned(),
        }
    );
}
