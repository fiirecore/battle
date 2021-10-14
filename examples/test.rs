use std::ops::Range;

use firecore_pokedex::{
    moves::{Move, MoveCategory, MoveTarget},
    pokemon::{
        owned::SavedPokemon, party::Party, stat::StatSet, Breeding, LearnableMove, Pokemon,
        Training,
    },
    types::PokemonType,
    BasicDex, Dex,
};
use rand::rngs::mock::StepRng;

use firecore_battle::prelude::*;

const POKEMON: Range<u16> = 0..3;

fn main() {
    let mut pokedex = BasicDex::default();
    let mut movedex = BasicDex::default();
    let itemdex = BasicDex::default();

    let move_id = "default".parse().unwrap();

    for id in POKEMON {
        pokedex.insert(Pokemon {
            id: id,
            name: format!("Test {}", id),
            primary_type: firecore_pokedex::types::PokemonType::Normal,
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

    let mut random = StepRng::new(34618, 3213);

    let party: Party<_> = POKEMON
        .into_iter()
        .flat_map(|id| {
            SavedPokemon::generate(&mut random, id, 30, None, None).init(
                &mut random,
                &pokedex,
                &movedex,
                &itemdex,
            )
        })
        .map(|mut o| {
            o.moves.add(None, &move_id);
            o
        })
        .collect();

    let players = vec![
        (
            1,
            BattleAi::new(StepRng::new(2351, 436246), party.clone()),
        ),
        (
            2,
            BattleAi::new(StepRng::new(35211, 46), party.clone()),
        ),
        (
            3,
            BattleAi::new(StepRng::new(123, 434626), party.clone()),
        ),
    ];

    let players = players.into_iter().map(|(id, player)| {
        BattlePlayer::<u8, BattleAi<StepRng, u8, 1>, 1>::new(
            id,
            Some(format!("Player {}", id)),
            player.party().clone(),
            Default::default(),
            player,
        )
    });

    let mut battle = Battle::new(BattleData::default(), players);

    let mut engine = DefaultMoveEngine::new::<StepRng>();

    while !battle.finished() {
        battle.update(&mut random, &mut engine, &itemdex);
    }

    println!("{} wins!", match battle.winner() {
        Some(id) => format!("Player #{}", id),
        None => "No one".to_owned(),
    });
}
