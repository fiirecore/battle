use std::ops::Range;

use firecore_pokedex::{
    moves::{owned::SavedMove, Move, MoveCategory, MoveTarget},
    pokemon::{
        owned::SavedPokemon, party::Party, stat::StatSet, Breeding, LearnableMove, Pokemon,
        Training,
    },
    types::PokemonType,
    BasicDex, Dex,
};
use rand::rngs::ThreadRng;

use firecore_battle::prelude::*;

const POKEMON: Range<u16> = 0..3;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        .init()
        .unwrap();

    let mut pokedex = BasicDex::default();
    let mut movedex = BasicDex::default();
    let itemdex = BasicDex::default();

    let move_id = "default".parse().unwrap();

    movedex.insert(Move {
        id: move_id,
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

    for id in POKEMON {
        pokedex.insert(Pokemon {
            id: id,
            name: format!("Test {}", id),
            primary_type: PokemonType::Normal,
            secondary_type: None,
            moves: vec![LearnableMove(0, move_id)],
            base: StatSet::uniform(70),
            species: "Test".to_owned(),
            height: 100,
            weight: 100,
            training: Training {
                base_exp: 80,
                growth_rate: Default::default(),
            },
            breeding: Breeding { gender: None },
        });
    }

    let mut random = rand::thread_rng();

    let party: Party<_> = POKEMON
        .into_iter()
        .map(|id| SavedPokemon::generate(&mut random, id, 30, None, None))
        .map(|mut o| {
            o.moves.push(SavedMove::from(move_id));
            o
        })
        .collect();

    let owned_party: Party<_> = party
        .iter()
        .cloned()
        .flat_map(|o| o.init(&mut random, &pokedex, &movedex, &itemdex))
        .collect();

    const AS: usize = 2;

    let mut players: Vec<_> = (1..100).into_iter().map(|_| BattleAi::<ThreadRng, u8, AS>::new(random.clone(), owned_party.clone())).collect();

    let mut battle = Battle::new(
        BattleData::default(),
        &mut random,
        &pokedex,
        &movedex,
        &itemdex,
        players.iter().enumerate().map(
            |(id, player)| PlayerWithEndpoint(LocalPlayer {
                id: id as _,
                name: Some(format!("Player {}", id)),
                party: party.clone(),
                settings: Default::default(),
            }, Box::new(player.endpoint()),
        )),
    );

    let mut engine = DefaultMoveEngine::new::<u8, ThreadRng>();

    engine.moves.insert(
        move_id,
        MoveExecution::Actions(vec![
            firecore_battle::host::engine::default::MoveUse::Damage(
                firecore_battle::moves::damage::DamageKind::Power(50),
            ),
        ]),
    );

    while !battle.finished() {
        battle.update(&mut random, &mut engine, &itemdex);
        for player in players.iter_mut() {
            if !player.finished() {
                player.update();
            }
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
