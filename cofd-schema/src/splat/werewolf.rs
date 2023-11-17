use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Renown {
	Purity,
	Glory,
	Honor,
	Wisdom,
	Cunning,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString)]
pub enum Auspice {
	Cahalith,
	Elodoth,
	Irraka,
	Ithaeur,
	Rahu,
}
