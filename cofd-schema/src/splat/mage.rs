use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, PartialEq, Eq)]
pub enum Arcanum {
	Death,
	Fate,
	Forces,
	Life,
	Matter,
	Mind,
	Prime,
	Space,
	Spirit,
	Time,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, PartialEq, Eq)]
pub enum Practice {
	Compelling,
	Knowing,
	Unveiling,

	Ruling,
	Shielding,
	Veiling,

	Fraying,
	Perfecting,
	Weaving,

	Patterning,
	Unraveling,

	Making,
	Unmaking,
}
