use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

pub mod attribute;
pub mod skill;

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Template {
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

#[derive(Debug, Serialize, Deserialize, EnumString, Display, AsRefStr)]
#[strum(ascii_case_insensitive)]
pub enum Trait {
	Speed,
	Defense,
	Initative,
	Perception,
	Health,
	Willpower,

	Beats,

	Size,
}
