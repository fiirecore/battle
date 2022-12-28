use pokedex::{moves::MoveId, pokemon::{Health, stat::StatType}, types::*};
use serde::{Deserialize, Serialize};

/// How powerful a [Move] is, in points. Some moves do not use power levels.
pub type Power = u8;
/// How accurate a [Move] is, in values 0 - 100.
pub type Accuracy = u8;
/// This determines whether the [Move] goes before another.
/// The higher the value, the higher the priority.
pub type Priority = i8;
/// This helps determine if a [Move] should be a critical hit.
/// The higher the value, the higher the chance of a critical hit.
/// This maxes out at 4.
pub type CriticalRate = u8;

pub type Critical = bool;
/// 0 through 100
pub type Percent = u8;

pub type MoveCancelId = tinystr::TinyAsciiStr<8>;
/// remove pokemon from ability to be active
pub type RemovePokemonId = tinystr::TinyAsciiStr<8>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BattleMove {
    pub id: MoveId,

    pub category: MoveCategory,

    /// The move's type.
    #[serde(rename = "type")]
    pub pokemon_type: PokemonType,
    /// If this is [None], the move will always land.
    pub accuracy: Option<Accuracy>,
    pub power: Option<Power>,

    #[serde(default)]
    pub priority: Priority,

    #[serde(default)]
    pub target: MoveTarget,

    /// If the move makes contact with the target.
    #[serde(default)]
    pub contact: Contact,

    /// Increments the chance of whether a move should critical hit or not.
    #[serde(default)]
    pub crit_rate: CriticalRate,
}

/// The category of a move.
// /// [MoveCategory::Physical] and [MoveCategory::Special] are usually for moves that deal damage.
// /// [Physical] deals physical damage ([Attack]) against a target pokemon's [Defense].
// /// [Special] deals special damage ([SpAttack]) against a target pokemon's [SpDefense].
// /// [MoveCategory::Status] moves usually afflict an ailment on a target pokemon or benefit the user pokemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum MoveCategory {
    Status,
    Physical,
    Special,
}

impl MoveCategory {
    /// Get a tuple of the attack and defense types of this category.
    pub fn stats(&self) -> (StatType, StatType) {
        (self.attack(), self.defense())
    }
    /// Get the attack type of this category.
    pub fn attack(&self) -> StatType {
        match self {
            MoveCategory::Physical => StatType::Attack,
            MoveCategory::Special => StatType::SpAttack,
            MoveCategory::Status => unreachable!("Cannot get attack stat for status move!"),
        }
    }
    /// Get the defense type of this category.
    pub fn defense(&self) -> StatType {
        match self {
            MoveCategory::Physical => StatType::Defense,
            MoveCategory::Special => StatType::SpDefense,
            MoveCategory::Status => unreachable!("Cannot get defense stat for status move!"),
        }
    }
}

/// The target of a [Move].
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum MoveTarget {
    Any,
    Ally,
    Allies,
    UserOrAlly,
    UserAndAllies,
    // UserOrAllies,
    User,
    Opponent,
    AllOpponents,
    RandomOpponent,
    AllOtherPokemon,
    AllPokemon,
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Contact(pub bool);

impl Default for Contact {
    fn default() -> Self {
        Self(true)
    }
}

impl Default for MoveTarget {
    fn default() -> Self {
        Self::None
    }
}

impl MoveTarget {
    pub fn needs_input(&self) -> bool {
        match self {
            MoveTarget::Ally | MoveTarget::Any | MoveTarget::Opponent | MoveTarget::UserOrAlly => {
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DamageKind {
    Power(Power),
    PercentCurrent(Percent),
    PercentMax(Percent),
    Constant(Health),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ClientDamage<N> {
    Result(DamageResult<N>),
    Number(N),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageResult<N> {
    /// Inflicted damage
    pub damage: N,
    /// Whether the attack was effective
    pub effective: Effective,
    /// If the attack was a critical hit
    pub crit: bool,
}

impl<N> ClientDamage<N> {
    pub fn damage(self) -> N {
        match self {
            ClientDamage::Result(result) => result.damage,
            ClientDamage::Number(n) => n,
        }
    }
}

impl<N: Default> Default for DamageResult<N> {
    fn default() -> Self {
        Self {
            damage: Default::default(),
            effective: Effective::Ineffective,
            crit: false,
        }
    }
}

impl<N> From<N> for DamageResult<N> {
    fn from(damage: N) -> Self {
        Self {
            damage,
            effective: Effective::Effective,
            crit: false,
        }
    }
}

impl MoveCategory {

    /// Test how [Effective] a [PokemonType] is on this pokemon, in a specified [MoveCategory].
    pub fn effective(&self, user: PokemonType, target: PokemonTypes) -> Effective {
        let primary = self.effective_table(user, target.primary);
        if let Some(secondary) = target.secondary {
            primary * self.effective_table(user, secondary)
        } else {
            primary
        }
    }

    /// Hardcoded effectiveness of a pokemon type on another pokemon type by move category.
    pub fn effective_table(&self, user: PokemonType, target: PokemonType) -> Effective {
        match self {
            MoveCategory::Status => Effective::Ineffective,
            _ => match user {
                PokemonType::Unknown => Effective::Effective,

                PokemonType::Normal => match target {
                    PokemonType::Ghost => Effective::Ineffective,
                    PokemonType::Rock | PokemonType::Steel => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Fire => match target {
                    PokemonType::Grass | PokemonType::Ice | PokemonType::Bug | PokemonType::Steel => Effective::SuperEffective,
                    PokemonType::Fire | PokemonType::Water | PokemonType::Rock | PokemonType::Dragon => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Water => match target {
                    PokemonType::Fire | PokemonType::Ground | PokemonType::Rock => Effective::SuperEffective,
                    PokemonType::Water | PokemonType::Grass | PokemonType::Dragon => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Electric => match target {
                    PokemonType::Water | PokemonType::Flying => Effective::SuperEffective,
                    PokemonType::Electric | PokemonType::Grass | PokemonType::Dragon => Effective::NotEffective,
                    PokemonType::Ground => Effective::Ineffective,
                    _ => Effective::Effective,
                },
                PokemonType::Grass => match target {
                    PokemonType::Water | PokemonType::Ground | PokemonType::Rock => Effective::SuperEffective,
                    PokemonType::Fire
                    | PokemonType::Grass
                    | PokemonType::Poison
                    | PokemonType::Flying
                    | PokemonType::Bug
                    | PokemonType::Dragon
                    | PokemonType::Steel => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Ice => match target {
                    PokemonType::Grass | PokemonType::Ground | PokemonType::Flying | PokemonType::Dragon => {
                        Effective::SuperEffective
                    }
                    PokemonType::Fire | PokemonType::Water | PokemonType::Ice | PokemonType::Steel => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Fighting => match target {
                    PokemonType::Normal | PokemonType::Ice | PokemonType::Rock | PokemonType::Dark | PokemonType::Steel => {
                        Effective::SuperEffective
                    }
                    PokemonType::Poison | PokemonType::Flying | PokemonType::Psychic | PokemonType::Bug | PokemonType::Fairy => {
                        Effective::NotEffective
                    }
                    PokemonType::Ghost => Effective::Ineffective,
                    _ => Effective::Effective,
                },
                PokemonType::Poison => match target {
                    PokemonType::Grass | PokemonType::Fairy => Effective::SuperEffective,
                    PokemonType::Poison | PokemonType::Ground | PokemonType::Rock | PokemonType::Ghost => {
                        Effective::NotEffective
                    }
                    PokemonType::Steel => Effective::Ineffective,
                    _ => Effective::Effective,
                },
                PokemonType::Ground => match target {
                    PokemonType::Fire | PokemonType::Electric | PokemonType::Poison | PokemonType::Rock | PokemonType::Steel => {
                        Effective::SuperEffective
                    }
                    PokemonType::Grass | PokemonType::Bug => Effective::NotEffective,
                    PokemonType::Flying => Effective::Ineffective,
                    _ => Effective::Effective,
                },
                PokemonType::Flying => match target {
                    PokemonType::Grass | PokemonType::Fighting | PokemonType::Bug => Effective::SuperEffective,
                    PokemonType::Electric | PokemonType::Rock | PokemonType::Steel => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Psychic => match target {
                    PokemonType::Fighting | PokemonType::Poison => Effective::SuperEffective,
                    PokemonType::Psychic | PokemonType::Steel => Effective::NotEffective,
                    PokemonType::Dark => Effective::Ineffective,
                    _ => Effective::Effective,
                },
                PokemonType::Bug => match target {
                    PokemonType::Grass | PokemonType::Psychic | PokemonType::Dark => Effective::SuperEffective,
                    PokemonType::Fire
                    | PokemonType::Fighting
                    | PokemonType::Poison
                    | PokemonType::Flying
                    | PokemonType::Ghost
                    | PokemonType::Steel
                    | PokemonType::Fairy => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Rock => match target {
                    PokemonType::Fire | PokemonType::Ice | PokemonType::Flying | PokemonType::Bug => Effective::SuperEffective,
                    PokemonType::Fighting | PokemonType::Ground | PokemonType::Steel => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Ghost => match target {
                    PokemonType::Psychic | PokemonType::Ghost => Effective::SuperEffective,
                    PokemonType::Dark => Effective::NotEffective,
                    PokemonType::Normal => Effective::Ineffective,
                    _ => Effective::Effective,
                },
                PokemonType::Dragon => match target {
                    PokemonType::Dragon => Effective::SuperEffective,
                    PokemonType::Steel => Effective::NotEffective,
                    PokemonType::Fairy => Effective::Ineffective,
                    _ => Effective::Effective,
                },
                PokemonType::Dark => match target {
                    PokemonType::Psychic | PokemonType::Ghost => Effective::SuperEffective,
                    PokemonType::Fighting | PokemonType::Dark | PokemonType::Fairy => Effective::NotEffective,
                    _ => Effective::Effective,
                },
                PokemonType::Steel => match target {
                    PokemonType::Ice | PokemonType::Rock | PokemonType::Fairy => Effective::SuperEffective,
                    PokemonType::Fire | PokemonType::Water | PokemonType::Electric | PokemonType::Steel => {
                        Effective::NotEffective
                    }
                    _ => Effective::Effective,
                },
                PokemonType::Fairy => match target {
                    PokemonType::Fighting | PokemonType::Dragon | PokemonType::Dark => Effective::SuperEffective,
                    PokemonType::Fire | PokemonType::Poison | PokemonType::Steel => Effective::NotEffective,
                    _ => Effective::Effective,
                },
            },
        }
    }
}
