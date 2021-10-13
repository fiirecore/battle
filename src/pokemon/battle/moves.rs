use rand::Rng;

use pokedex::{
    moves::{owned::OwnedMove, CriticalRate, Move, MoveCategory, MoveId, Power},
    pokemon::Health,
    types::{Effective, PokemonType},
};

use crate::moves::{
    damage::{DamageKind, DamageResult},
    engine::MoveEngine,
    target::TargetLocation,
    MoveResult, MoveUse,
};

impl<'d> super::BattlePokemon<'d> {
    // To - do: uses PP on use
    pub fn use_own_move<R: Rng + Clone + 'static, E: MoveEngine>(
        &self,
        random: &mut R,
        engine: &mut E,
        move_index: usize,
        targets: Vec<(TargetLocation, &Self)>,
    ) -> Option<(MoveId, Vec<(TargetLocation, MoveResult)>)> {
        let m = self
            .moves
            .get(move_index)
            .map(OwnedMove::try_use)
            .flatten()?;

        let results = engine
            .execute(random, &m, self, targets)
            .unwrap_or_else(|err| {
                log::error!("Could not use move {} with error {}", m.name, err);
                vec![(TargetLocation::User, MoveResult::Error)]
            });

        Some((m.id, results))
    }

    pub fn move_usage<R: Rng>(
        &self,
        random: &mut R,
        results: &mut Vec<(TargetLocation, MoveResult)>,
        actions: &Vec<MoveUse>,
        m: &Move,
        target: (TargetLocation, &Self),
    ) {
        for action in actions {
            match action {
                MoveUse::Damage(kind) => {
                    results.push((
                        target.0,
                        MoveResult::Damage(target.1.damage_kind(
                            random,
                            target.1,
                            *kind,
                            m.category,
                            m.pokemon_type,
                            m.crit_rate,
                        )),
                    ));
                }
                MoveUse::Ailment(status, length, chance) => {
                    if target.1.ailment.is_none() {
                        if random.gen_bool(*chance as f64 / 100.0) {
                            results.push((
                                target.0,
                                MoveResult::Ailment(length.init(*status, random)),
                            ));
                        }
                    }
                }
                MoveUse::Drain(kind, percent) => {
                    let result = self.damage_kind(
                        random,
                        target.1,
                        *kind,
                        m.category,
                        m.pokemon_type,
                        m.crit_rate,
                    );

                    let healing = (result.damage as f32 * *percent as f32 / 100.0) as i16;

                    results.push((target.0, MoveResult::Damage(result)));
                    results.push((TargetLocation::User, MoveResult::Heal(healing)))
                }
                MoveUse::Stat(stat, stage) => {
                    if target.1.stages.can_change(*stat, *stage) {
                        results.push((target.0, MoveResult::Stat(*stat, *stage)));
                    }
                }
                // MoveUseType::Linger(..) => {
                // 	results.insert(target.instance, Some(MoveAction::Todo));
                // }
                MoveUse::Flinch => results.push((target.0, MoveResult::Flinch)),
                MoveUse::Chance(actions, chance) => {
                    if random.gen_range(0..=100) < *chance {
                        self.move_usage(random, results, actions, m, target);
                    }
                }
            }
        }
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
        let crit = Self::crit(random, crit_rate);

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
            Self::damage_range(random),
        )
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

    pub fn damage_range(random: &mut impl Rng) -> u8 {
        random.gen_range(85..=100u8)
    }

    pub fn move_power_damage(
        &self,
        target: &Self,
        power: Power,
        category: MoveCategory,
        move_type: PokemonType,
        crit: bool,
        damage_range: u8,
    ) -> DamageResult<Health> {
        let effective = target.pokemon.effective(move_type, category);
        let (attack, defense) = category.stats();
        let attack = self.stat(attack);
        let defense = target.stat(defense);
        if effective == Effective::Ineffective {
            return DamageResult::default();
        }
        let damage =
            (((((2.0 * self.level as f64 / 5.0 + 2.0).floor() * attack as f64 * power as f64
                / defense as f64)
                .floor()
                / 50.0)
                .floor()
                * effective.multiplier() as f64)
                + 2.0)
                * (damage_range as f64 / 100.0)
                * if self.pokemon.primary_type == move_type {
                    1.5
                } else {
                    1.0
                }
                * if crit { 1.5 } else { 1.0 };
        let damage = damage.min(u16::MAX as f64) as u16;
        DamageResult {
            damage,
            effective,
            crit,
        }
    }
}
