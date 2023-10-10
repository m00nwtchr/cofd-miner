use serde::{Deserialize, Serialize};
use strum::EnumString;

pub mod attribute;
pub mod skill;

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Template {
	#[strum(serialize = "Mage", serialize = "Awakened")]
	Mage,
	#[strum(serialize = "Vampire", serialize = "Kindred")]
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

#[derive(Debug, Serialize, Deserialize, EnumString)]
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
pub enum Trait {
	Willpower,
	// Attribute(Attribute),
	// Skill(Skill),
}
