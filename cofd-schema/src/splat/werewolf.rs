use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
pub enum Renown {
	Purity,
	Glory,
	Honor,
	Wisdom,
	Cunning,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, PartialEq, Eq)]
pub enum Auspice {
	Cahalith,
	Elodoth,
	Irraka,
	Ithaeur,
	Rahu,
}
