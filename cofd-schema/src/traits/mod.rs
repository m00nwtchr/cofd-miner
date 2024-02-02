use std::{convert::Into, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, ParseError};

use self::{
	attribute::{Attribute, MentalAttribute, PhysicalAttribute, SocialAttribute},
	skill::{Skill, SkillMarker},
};

pub mod attribute;
pub mod skill;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
pub enum Template {
	#[strum(to_string = "Mortal", serialize = "Human")]
	Mortal,
	#[strum(to_string = "Mage", serialize = "Awakened")]
	Mage,
	#[strum(to_string = "Vampire", serialize = "Kindred")]
	Vampire,
	Werewolf,
	Promethean,
	Changeling,
	Hunter,
	Bound,
	Mummy,
	Demon,
	Beast,
	Deviant,

	// Mage
	Sleepwalker,
	Proximi,
	// Vampire
	Ghoul,
	// Werewolf
	#[strum(to_string = "Wolf-Blooded")]
	WolfBlooded,
	// Changeling
	#[strum(to_string = "Fae-Blooded")]
	FaeTouched,
	// Mummy
	Endless,
	// Demon
	#[strum(to_string = "Demon-Blooded")]
	DemonBlooded,
	Stigmatic,
}

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Eq, Display,
)]
#[strum(ascii_case_insensitive)]
pub enum SupernaturalTolerance {
	Gnosis,
	#[strum(to_string = "Blood Potency")]
	BloodPotency,
	#[strum(to_string = "Primal Urge")]
	PrimalUrge,
	Azoth,
	Wyrd,
	Synergy,
	Sekhem,
	Primium,
	Lair,
}

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Fuel {
	Mana,
	Vitae,
	Essence,
	Pyros,
	Glamour,
	Plasm,
	Pillar,
	Aether,
	// Satiety,
}

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Integrity {
	Integrity,
	Wisdom,
	Humanity,
	Harmony,
	Pilgrimage,
	Clarity,
	Memory,
	Cover,
	Satiety,
	Instability,
}

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, AsRefStr, PartialEq, Eq,
)]
#[strum(ascii_case_insensitive)]
pub enum DerivedTrait {
	Speed,
	Defense,
	Initative,
	Perception,
	Health,
	Willpower,

	Beats,

	Size,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, derive_more::Display)]
#[serde(untagged)]
pub enum Trait {
	Attribute(Attribute),
	Skill(Skill),

	DerivedTrait(DerivedTrait),

	SupernaturalTolerance(SupernaturalTolerance),
}

impl From<MentalAttribute> for Trait {
	fn from(value: MentalAttribute) -> Self {
		Self::Attribute(value.into())
	}
}
impl From<PhysicalAttribute> for Trait {
	fn from(value: PhysicalAttribute) -> Self {
		Self::Attribute(value.into())
	}
}
impl From<SocialAttribute> for Trait {
	fn from(value: SocialAttribute) -> Self {
		Self::Attribute(value.into())
	}
}

impl From<Attribute> for Trait {
	fn from(value: Attribute) -> Self {
		Self::Attribute(value)
	}
}

impl<T: Into<Skill> + SkillMarker> From<T> for Trait {
	fn from(value: T) -> Self {
		Self::Skill(value.into())
	}
}

impl From<DerivedTrait> for Trait {
	fn from(value: DerivedTrait) -> Self {
		Self::DerivedTrait(value)
	}
}

impl From<SupernaturalTolerance> for Trait {
	fn from(value: SupernaturalTolerance) -> Self {
		Self::SupernaturalTolerance(value)
	}
}

impl FromStr for Trait {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Attribute::from_str(s)
			.map(Into::into)
			.or_else(|_| Skill::from_str(s).map(Into::into))
			.or_else(|_| SupernaturalTolerance::from_str(s).map(Into::into))
			.or_else(|_| DerivedTrait::from_str(s).map(Into::into))
	}
}

impl AsRef<str> for Trait {
	fn as_ref(&self) -> &str {
		match self {
			Trait::Attribute(attr) => attr.as_ref(),
			Trait::Skill(skill) => skill.as_ref(),
			Trait::DerivedTrait(dt) => dt.as_ref(),
			Trait::SupernaturalTolerance(st) => st.as_ref(),
		}
	}
}
