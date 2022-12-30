use core::ops::{Deref, DerefMut};
use hashbrown::HashSet;
use rand::Rng;
use serde::{Deserialize, Serialize};

use pokedex::{
    moves::MoveId,
    pokemon::{
        owned::{OwnedPokemon, SavedPokemon},
        stat::{BaseStat, StatType},
        Experience, Health,
    },
    types::{Effective, PokemonType},
};

use crate::{
    data::VersusType,
    moves::{Accuracy, CriticalRate, DamageKind, DamageResult, MoveCategory, Percent, Power},
};

pub mod remote;
pub mod stat;

pub type ActivePosition = usize;
pub type PartyPosition = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct TeamIndex<ID>(pub ID, pub usize);

impl<ID> TeamIndex<ID> {
    pub fn team(&self) -> &ID {
        &self.0
    }

    pub fn index(&self) -> usize {
        self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Indexed<ID, T>(pub TeamIndex<ID>, pub T);

impl<ID: core::fmt::Display> core::fmt::Display for TeamIndex<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} #{}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PokemonView {
    Partial(remote::RemotePokemon),
    Full(SavedPokemon),
}

pub trait PokemonInstance {
    // fn id(&self) -> &PokemonId;

    fn fainted(&self) -> bool;
}

/// To - do: factor in accuracy
pub fn throw_move<R: rand::Rng>(random: &mut R, accuracy: Option<Accuracy>) -> bool {
    accuracy
        .map(|accuracy| random.gen_range(0..100) < accuracy)
        .unwrap_or(true)
}

pub fn crit(random: &mut impl Rng, crit_rate: CriticalRate) -> bool {
    random.gen_bool(match crit_rate {
        0 => 0.0625, // 1 / 16
        1 => 0.125,  // 1 / 8
        2 => 0.25,   // 1 / 4
        3 => 1.0 / 3.0,
        _ => 0.5, // rates 4 and above, 1 / 2
    })
}

pub fn damage_range(random: &mut impl Rng) -> Percent {
    random.gen_range(85..=100u8)
}

#[derive(Debug, Clone)]
pub struct BattlePokemon {
    pub p: OwnedPokemon,
    pub stages: stat::StatStages,
    pub(crate) learnable: HashSet<MoveId>,
    revealed: bool,
}

impl<P> PokemonInstance for Option<remote::UnknownPokemon<P>> {
    // fn id(&self) -> &PokemonId {
    //     self.as_ref()
    //         .map(|u| u.pokemon.as_id())
    //         .unwrap_or(&Pokemon::UNKNOWN)
    // }

    fn fainted(&self) -> bool {
        self.as_ref().map(|u| u.fainted()).unwrap_or_default()
    }
}

impl<P> PokemonInstance for remote::UnknownPokemon<P> {
    // fn id(&self) -> &PokemonId {
    //     self.pokemon.as_id()
    // }

    fn fainted(&self) -> bool {
        remote::UnknownPokemon::fainted(self)
    }
}

impl PokemonInstance for OwnedPokemon {
    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }

    // fn id(&self) -> &PokemonId {
    //     &self.pokemon.id
    // }
}

impl BattlePokemon {
    // pub fn try_flinch(&mut self) -> bool {
    //     if self.flinch {
    //         self.flinch = false;
    //         true
    //     } else {
    //         false
    //     }
    // }

    pub fn battle_exp_from(&self, versus: VersusType) -> Experience {
        let experience = self.exp_from();
        let experience = match versus.is_wild() {
            true => experience.saturating_mul(3) / 2,
            false => experience,
        };

        #[cfg(debug_assertions)]
        let experience = experience.saturating_mul(7);

        experience
    }

    pub fn stat(&self, stat: StatType) -> BaseStat {
        stat::StatStages::mult(
            self.p.stat(stat),
            self.stages[stat::BattleStatType::Basic(stat)],
        )
    }

    pub fn damage_kind(
        &self,
        random: &mut impl Rng,
        target: &Self,
        kind: DamageKind,
        category: MoveCategory,
        move_type: PokemonType,
        crit_rate: CriticalRate,
    ) -> DamageResult<Health> {
        let effective = category.effective(move_type, target.pokemon.types);
        let crit = crit(random, crit_rate);

        if let DamageKind::Power(power) = kind {
            self.move_power_damage_random(random, target, power, category, move_type, crit)
        } else {
            DamageResult {
                damage: match matches!(effective, Effective::Ineffective) {
                    true => 0,
                    false => match kind {
                        DamageKind::PercentCurrent(percent) => {
                            (target.hp() as f32 * effective.multiplier() * percent as f32 / 100.0)
                                as Health
                        }
                        DamageKind::PercentMax(percent) => {
                            (target.max_hp() as f32 * effective.multiplier() * percent as f32
                                / 100.0) as Health
                        }
                        DamageKind::Constant(damage) => damage,
                        DamageKind::Power(..) => unreachable!(),
                    },
                },
                effective,
                crit,
            }
        }
    }

    pub fn move_power_damage_random(
        &self,
        random: &mut impl Rng,
        target: &Self,
        power: Power,
        category: MoveCategory,
        move_type: PokemonType,
        crit: bool,
    ) -> DamageResult<Health> {
        self.move_power_damage(
            target,
            power,
            category,
            move_type,
            crit,
            damage_range(random),
        )
    }

    pub fn move_power_damage(
        &self,
        target: &Self,
        power: Power,
        category: MoveCategory,
        move_type: PokemonType,
        crit: bool,
        range: u8,
    ) -> DamageResult<Health> {
        let effective = category.effective(move_type, target.pokemon.types);
        let (attack, defense) = category.stats();
        let attack = self.stat(attack);
        let defense = target.stat(defense);
        if matches!(effective, Effective::Ineffective) {
            return DamageResult::default();
        }

        fn crit_dmg(crit: bool) -> f64 {
            match crit {
                true => 1.5,
                false => 1.0,
            }
        }

        let mut damage = 2.0 * self.level as f64;
        damage /= 5.0;
        damage += 2.0;
        damage = damage.floor();
        damage *= power as f64;
        damage *= attack as f64 / defense as f64;
        damage = damage.floor();
        damage /= 50.0;
        damage = damage.floor();
        damage += 2.0;

        damage *= range as f64 / 100.0;
        damage *= crit_dmg(self.pokemon.types.primary == move_type);
        damage *= crit_dmg(crit);
        damage *= effective.multiplier() as f64;

        // println!(
        //     "PWR: {}, LVL: {}, ATK: {}, DEF: {}, DMG: {}",
        //     power, self.level, attack, defense, damage
        // );

        DamageResult {
            damage: damage.round() as _,
            effective,
            crit,
        }
    }

    pub fn reveal(&mut self) {
        self.revealed = true;
    }

    pub fn get_revealed(&self) -> Option<remote::RemotePokemon> {
        self.revealed
            .then(|| remote::UnknownPokemon::new(&self.p).uninit())
    }

    pub fn try_learn_moves(&mut self, moves: impl IntoIterator<Item = MoveId>) {
        self.learnable.extend(moves)
    }
}

impl From<OwnedPokemon> for BattlePokemon {
    fn from(p: OwnedPokemon) -> Self {
        Self {
            p,
            stages: Default::default(),
            learnable: Default::default(),
            revealed: false,
        }
    }
}

impl Deref for BattlePokemon {
    type Target = OwnedPokemon;

    fn deref(&self) -> &Self::Target {
        &self.p
    }
}

impl DerefMut for BattlePokemon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.p
    }
}

impl core::fmt::Display for BattlePokemon {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\"{}\": {}, {}/{} HP",
            self.name(),
            self.level,
            self.hp(),
            self.max_hp()
        )
    }
}

impl PokemonInstance for BattlePokemon {
    // fn id(&self) -> &PokemonId {
    //     &self.pokemon.id
    // }

    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}


#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use firecore_pokedex::{
        moves::set::OwnedMoveSet,
        pokemon::{
            data::{Breeding, Gender, GrowthRate, Training},
            owned::OwnedPokemon,
            stat::{StatSet, StatType},
            Nature, Pokemon, PokemonId,
        },
        stat_set,
        types::{PokemonType, PokemonTypes},
    };

    use crate::moves::MoveCategory;

    use super::BattlePokemon;

    #[test]
    fn damage() {
        let feraligatr = Arc::new(Pokemon {
            id: PokemonId(160),
            name: "Feraligatr".to_owned(),
            types: PokemonTypes {
                primary: PokemonType::Water,
                secondary: None,
            },
            moves: vec![],
            base: stat_set! {
                StatType::Health => 85,
                StatType::Attack => 105,
                StatType::Defense => 100,
                StatType::SpAttack => 79,
                StatType::SpDefense => 83,
                StatType::Speed => 78,
            },
            species: "Big Jaw".to_owned(),
            evolution: None,
            height: 23,
            weight: 888,
            training: Training {
                base_exp: 239,
                growth: GrowthRate::MediumSlow,
            },
            breeding: Breeding { gender: Some(6) },
        });

        let geodude = Arc::new(Pokemon {
            id: PokemonId(74),
            name: "Geodude".to_owned(),
            types: PokemonTypes {
                primary: PokemonType::Rock,
                secondary: Some(PokemonType::Ground),
            },
            moves: vec![],
            base: stat_set! {
                StatType::Health => 40,
                StatType::Attack =>  80,
                StatType::Defense => 100,
                StatType::SpAttack => 30,
                StatType::SpDefense => 30,
                StatType::Speed =>  20,
            },
            species: "Rock".to_owned(),
            evolution: None,
            height: 4,
            weight: 20,
            training: Training {
                base_exp: 60,
                growth: GrowthRate::MediumSlow,
            },
            breeding: Breeding { gender: Some(3) },
        });

        let mut user = OwnedPokemon {
            pokemon: feraligatr,
            level: 50,
            gender: Gender::Male,
            nature: Nature::Adamant,
            hp: 0,
            ivs: StatSet::uniform(15),
            evs: StatSet::uniform(50),
            friendship: Pokemon::default_friendship(),
            ailment: None,
            nickname: None,
            moves: OwnedMoveSet::default(),
            item: None,
            experience: 0,
        };

        user.heal_hp(None);

        let mut target = OwnedPokemon {
            pokemon: geodude,
            level: 10,
            gender: Gender::Female,
            nature: Nature::Hardy,
            hp: 0,
            ivs: StatSet::uniform(0),
            evs: StatSet::uniform(0),
            friendship: Pokemon::default_friendship(),
            ailment: None,
            nickname: None,
            moves: OwnedMoveSet::default(),
            item: None,
            experience: 0,
        };

        target.heal_hp(None);

        let user = BattlePokemon::from(user);

        let target = target.into();

        let damage = user
            .move_power_damage(
                &target,
                80,
                MoveCategory::Physical,
                PokemonType::Water,
                false,
                100,
            )
            .damage;
        assert!(damage <= 1200, "Damage passed threshold! {} > 1200", damage);
        assert!(
            damage >= 1100,
            "Damage could not reach threshold! {} < 1100",
            damage
        );
    }
}
