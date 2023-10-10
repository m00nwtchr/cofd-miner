use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum MentalAttribute {
	Intelligence,
	Wits,
	Resolve,
}

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum PhysicalAttribute {
	Strength,
	Dexterity,
	Stamina,
}

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum SocialAttribute {
	Presence,
	Manipulation,
	Composure,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Attribute {
	Mental(MentalAttribute),
	Physical(PhysicalAttribute),
	Social(SocialAttribute),
}

impl FromStr for Attribute {
	type Err = strum::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		MentalAttribute::from_str(s)
			.map(Into::into)
			.or_else(|_| PhysicalAttribute::from_str(s).map(Into::into))
			.or_else(|_| SocialAttribute::from_str(s).map(Into::into))
	}
}

impl From<MentalAttribute> for Attribute {
	fn from(value: MentalAttribute) -> Self {
		Attribute::Mental(value)
	}
}

impl From<PhysicalAttribute> for Attribute {
	fn from(value: PhysicalAttribute) -> Self {
		Attribute::Physical(value)
	}
}

impl From<SocialAttribute> for Attribute {
	fn from(value: SocialAttribute) -> Self {
		Attribute::Social(value)
	}
}
