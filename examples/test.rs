use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use firecore_pokedex::{
    moves::Move,
    pokemon::{
        owned::SavedPokemon, party::Party, stat::StatSet, Breeding, LearnableMove, Pokemon,
        Training,
    },
    BasicDex, Dex, Identifiable,
};
use rand::rngs::mock::StepRng;

use firecore_battle::{
    message::{ClientMessage, ServerMessage},
    moves::engine::DefaultMoveEngine,
    player::{ai::BattlePlayerAi, BattlePlayer},
    Battle, BattleData, BattleEndpoint, BattleType,
};

fn main() {
    let mut pokedex = BasicDex::default();

    let mid = "default".parse().unwrap();

    for i in 0..3 {
        pokedex.insert(Pokemon {
            id: i,
            name: format!("number {}", i),
            primary_type: firecore_pokedex::types::PokemonType::Normal,
            secondary_type: None,
            moves: vec![LearnableMove(0, mid)],
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

    // let pokedex = Rc::new(pokedex);

    let mut movedex = BasicDex::default();

    movedex.insert(Move {
        id: mid,
        name: "Test Move".to_owned(),
        category: firecore_pokedex::moves::MoveCategory::Physical,
        pokemon_type: firecore_pokedex::types::PokemonType::Normal,
        accuracy: Some(50),
        power: Some(50),
        pp: 50,
        priority: 0,
        target: firecore_pokedex::moves::MoveTarget::Opponent,
        contact: false,
        crit_rate: 0,
    });

    #[derive(Clone)]
    struct BattleP<'d>(Rc<RefCell<BattlePlayerAi<'d, StepRng, u8>>>);

    impl<'d> BattleEndpoint<u8> for BattleP<'d> {
        fn send(&mut self, message: ServerMessage<u8>) {
            self.0.try_borrow_mut().unwrap().send(message)
        }

        fn receive(&mut self) -> Option<ClientMessage> {
            self.0.try_borrow_mut().unwrap().receive()
        }
    }

    let itemdex = BasicDex::default();

    let mut r = StepRng::new(34618, 3213);

    let mut party = Party::default();

    for i in 0..3 {
        let p = SavedPokemon::generate(&mut r, i, 30, None, None);
        if let Some(mut p) = p.init(&mut r, &pokedex, &movedex, &itemdex) {
            p.moves.add(None, &mid);
            party.push(p)
        }
    }

    let (p1, p2, p3) = (
        BattlePlayerAi::new(StepRng::new(2351, 436246), party.clone()),
        BattlePlayerAi::new(StepRng::new(35211, 46), party.clone()),
        BattlePlayerAi::new(StepRng::new(123, 434626), party.clone())
    );

    let bp1 = BattlePlayer::new(
        0,
        p1.party().clone(),
        Some("P1".to_owned()),
        Default::default(),
        p1,
        1,
    );
    let bp2 = BattlePlayer::new(
        1,
        p2.party().clone(),
        Some("P2".to_owned()),
        Default::default(),
        p2,
        1,
    );
    let bp3 = BattlePlayer::new(
        2,
        p3.party().clone(),
        Some("P3".to_owned()),
        Default::default(),
        p3,
        1,
    );

    let mut battle = Battle::new(
        BattleData {
            type_: BattleType::Trainer,
        },
        vec![bp1, bp2, bp3].into_iter(),
    );

    let mut engine = DefaultMoveEngine::new::<StepRng>();

    while !battle.finished() {
        battle.update(&mut r, &mut engine, &itemdex);
    }

    println!("{:?} wins!", battle.winner());
}
