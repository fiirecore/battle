use core::ops::{Deref, DerefMut};
use hashbrown::HashSet;
use rand::Rng;

use pokedex::{
    moves::{CriticalRate, Move, MoveCategory, MoveId, Power},
    pokemon::{
        owned::OwnedPokemon,
        stat::{BaseStat, StatType},
        Experience, Health,
    },
    types::{Effective, PokemonType},
    Uninitializable,
};

use crate::{
    moves::{
        damage::{DamageKind, DamageResult},
        BattleMove,
    },
    party::PartyIndex,
    pokemon::{
        remote::{RemotePokemon, UnknownPokemon},
        stat::{BattleStatType, StatStages},
        PokemonView,
    },
    BattleType,
};

#[derive(Clone, Copy)]
pub struct ActivePokemon<ID> {
    pub index: usize,
    pub queued_move: Option<BattleMove<ID>>,
}

impl<ID> ActivePokemon<ID> {
    pub fn into_remote<const AS: usize>(this: &[Option<Self>; AS]) -> [Option<usize>; AS] {
        let mut active = [None; AS];

        for (i, a) in this.iter().enumerate() {
            active[i] = a.as_ref().map(PartyIndex::index);
        }

        return active;
    }

    pub fn queued(&self) -> bool {
        self.queued_move.is_some()
    }
}

pub struct BattlePokemon<'d> {
    pub instance: OwnedPokemon<'d>,
    pub learnable_moves: HashSet<MoveId>,
    // pub persistent: Option<PersistentMove>,
    pub caught: bool,
    pub known: bool,
    pub flinch: bool,
    pub requestable: bool,
    pub stages: StatStages,
}

impl<'d> BattlePokemon<'d> {
    pub fn know(&mut self) -> Option<RemotePokemon> {
        (!self.known).then(|| {
            self.known = true;
            UnknownPokemon::new(&self.instance).uninit()
        })
    }

    pub fn try_flinch(&mut self) -> bool {
        if self.flinch {
            self.flinch = false;
            true
        } else {
            false
        }
    }

    // To - do: factor in accuracy
    pub fn throw_move<R: rand::Rng>(&self, random: &mut R, m: &Move) -> bool {
        m.accuracy
            .map(|accuracy| random.gen_range(0..100) < accuracy)
            .unwrap_or(true)
    }

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
            self.instance.stat(stat),
            self.stages.get(BattleStatType::Basic(stat)),
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

impl<ID> PartyIndex for ActivePokemon<ID> {
    fn index(&self) -> usize {
        self.index
    }
}

impl<ID> From<usize> for ActivePokemon<ID> {
    fn from(index: usize) -> Self {
        Self {
            index,
            queued_move: None,
        }
    }
}

impl<ID> core::fmt::Debug for ActivePokemon<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivePokemon")
            .field("index", &self.index)
            .field("queued_move", &self.queued_move.is_some())
            .finish()
    }
}

impl<'d> From<OwnedPokemon<'d>> for BattlePokemon<'d> {
    fn from(instance: OwnedPokemon<'d>) -> Self {
        Self {
            instance,
            learnable_moves: Default::default(),
            caught: false,
            known: false,
            flinch: false,
            requestable: false,
            stages: Default::default(),
        }
    }
}

impl<'d> PokemonView for BattlePokemon<'d> {
    fn fainted(&self) -> bool {
        OwnedPokemon::fainted(self)
    }
}

impl<'d> Deref for BattlePokemon<'d> {
    type Target = OwnedPokemon<'d>;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl<'d> DerefMut for BattlePokemon<'d> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}

impl<'d> core::fmt::Debug for BattlePokemon<'d> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
