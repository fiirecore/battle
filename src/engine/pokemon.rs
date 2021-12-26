use core::ops::{Deref, DerefMut};

use pokedex::{
    item::Item,
    moves::{Accuracy, CriticalRate, Move, MoveCategory, Power},
    pokemon::{
        owned::OwnedPokemon,
        stat::{BaseStat, StatType},
        Experience, Health, Pokemon,
    },
    types::{Effective, PokemonType},
};
use rand::Rng;

use crate::{
    data::BattleType,
    moves::{
        damage::{DamageKind, DamageResult},
        Percent,
    },
    pokemon::stat::{BattleStatType, StatStages},
};

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
pub struct BattlePokemon<
    P: Deref<Target = Pokemon>,
    M: Deref<Target = Move>,
    I: Deref<Target = Item>,
> {
    pub p: OwnedPokemon<P, M, I>,
    // pub persistent: Option<PersistentMove>,
    pub stages: StatStages,
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>>
    BattlePokemon<P, M, I>
{
    // pub fn try_flinch(&mut self) -> bool {
    //     if self.flinch {
    //         self.flinch = false;
    //         true
    //     } else {
    //         false
    //     }
    // }

    pub fn battle_exp_from(&self, type_: &BattleType) -> Experience {
        let experience = self.exp_from();
        let experience = match matches!(type_, BattleType::Wild) {
            true => experience.saturating_mul(3) / 2,
            false => experience,
        };

        #[cfg(debug_assertions)]
        let experience = experience.saturating_mul(7);

        experience
    }

    pub fn stat(&self, stat: StatType) -> BaseStat {
        StatStages::mult(
            self.p.stat(stat),
            self.stages.get(BattleStatType::Basic(stat)),
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
        let effective = target.pokemon.effective(move_type, category);
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
        let effective = target.pokemon.effective(move_type, category);
        let (attack, defense) = category.stats();
        let attack = self.stat(attack);
        let defense = target.stat(defense);
        if matches!(effective, Effective::Ineffective) {
            return DamageResult::default();
        }

        /// Same type attack bonus
        fn stab(t1: PokemonType, t2: PokemonType) -> f64 {
            crit_dmg(t1 == t2)
        }

        fn crit_dmg(crit: bool) -> f64 {
            match crit {
                true => 1.5,
                false => 1.0,
            }
        }

        let mut e_mult = move_type.effective(target.pokemon.primary_type, category).multiplier();
        if let Some(secondary) = target.pokemon.secondary_type {
            e_mult *= move_type.effective(secondary, category).multiplier();
        }
        let e_mult = e_mult as f64;

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
        damage *= stab(self.pokemon.primary_type, move_type);
        damage *= crit_dmg(crit);
        damage *= e_mult;

        println!("PWR: {}, LVL: {}, ATK: {}, DEF: {}, DMG: {}", power, self.level, attack, defense, damage);

        DamageResult {
            damage: damage.round() as _,
            effective,
            crit,
        }
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>>
    From<OwnedPokemon<P, M, I>> for BattlePokemon<P, M, I>
{
    fn from(p: OwnedPokemon<P, M, I>) -> Self {
        Self {
            p,
            stages: Default::default(),
        }
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>> Deref
    for BattlePokemon<P, M, I>
{
    type Target = OwnedPokemon<P, M, I>;

    fn deref(&self) -> &Self::Target {
        &self.p
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I: Deref<Target = Item>> DerefMut
    for BattlePokemon<P, M, I>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.p
    }
}

#[cfg(test)]
mod tests {

    use firecore_pokedex::{
        pokemon::{
            data::{Breeding, Gender, GrowthRate, Training},
            nature::Nature,
            owned::OwnedPokemon,
            stat::StatSet,
            Pokemon,
        },
        types::PokemonType, item::Item, moves::{set::OwnedMoveSet, Move, MoveCategory},
    };

    use super::BattlePokemon;

    #[test]
    fn damage() {
        let feraligatr = Pokemon {
            id: 160,
            name: "Feraligatr".to_owned(),
            primary_type: PokemonType::Water,
            secondary_type: None,
            moves: vec![],
            base: StatSet {
                hp: 85,
                atk: 105,
                def: 100,
                sp_atk: 79,
                sp_def: 83,
                speed: 78,
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
        };

        let geodude = Pokemon {
            id: 74,
            name: "Geodude".to_owned(),
            primary_type: PokemonType::Rock,
            secondary_type: Some(PokemonType::Ground),
            moves: vec![],
            base: StatSet {
                hp: 40,
                atk: 80,
                def: 100,
                sp_atk: 30,
                sp_def: 30,
                speed: 20,
            },
            species: "Rock".to_owned(),
            evolution: None,
            height: 0_4,
            weight: 20,
            training: Training { base_exp: 60, growth: GrowthRate::MediumSlow },
            breeding: Breeding { gender: Some(3) },
        };

        let mut user = OwnedPokemon {
            pokemon: &feraligatr,
            level: 50,
            gender: Gender::Male,
            nature: Nature::Adamant,
            hp: 0,
            ivs: StatSet::uniform(15),
            evs: StatSet::uniform(50),
            friendship: Pokemon::default_friendship(),
            ailment: None,
            nickname: None,
            moves: OwnedMoveSet::<&Move>::default(),
            item: Option::<&Item>::None,
            experience: 0,
        };

        user.heal_hp(None);

        let mut target = OwnedPokemon {
            pokemon: &geodude,
            level: 10,
            gender: Gender::Female,
            nature: Nature::Hardy,
            hp: 0,
            ivs: StatSet::uniform(0),
            evs: StatSet::uniform(0),
            friendship: Pokemon::default_friendship(),
            ailment: None,
            nickname: None,
            moves: OwnedMoveSet::<&Move>::default(),
            item: Option::<&Item>::None,
            experience: 0,
        };

        target.heal_hp(None);

        let user = BattlePokemon {
            p: user,
            stages: Default::default(),
        };

        let target = target.into();

        let damage = user.move_power_damage(&target, 80, MoveCategory::Physical, PokemonType::Water, false, 100).damage;
        assert!(damage <= 1200, "Damage overreached threshold! {} > 1200", damage);
        assert!(damage >= 1100, "Damage could not reach threshold! {} < 1100", damage);

    }
}
