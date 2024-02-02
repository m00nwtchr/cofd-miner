use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIs, EnumString};

pub(crate) trait AttributeMarker {}

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, AsRefStr, PartialEq, Eq,
)]
#[strum(ascii_case_insensitive)]
pub enum MentalAttribute {
	Intelligence,
	Wits,
	Resolve,
}

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, AsRefStr, PartialEq, Eq,
)]
#[strum(ascii_case_insensitive)]
pub enum PhysicalAttribute {
	Strength,
	Dexterity,
	Stamina,
}

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, AsRefStr, PartialEq, Eq,
)]
#[strum(ascii_case_insensitive)]
pub enum SocialAttribute {
	Presence,
	Manipulation,
	Composure,
}

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, EnumIs, PartialEq, Eq, derive_more::Display,
)]
#[serde(untagged)]
pub enum Attribute {
	Mental(MentalAttribute),
	Physical(PhysicalAttribute),
	Social(SocialAttribute),
}

impl AttributeMarker for MentalAttribute {}
impl AttributeMarker for PhysicalAttribute {}
impl AttributeMarker for SocialAttribute {}
impl AttributeMarker for Attribute {}

impl AsRef<str> for Attribute {
	fn as_ref(&self) -> &str {
		match self {
			Self::Mental(attr) => attr.as_ref(),
			Self::Physical(attr) => attr.as_ref(),
			Self::Social(attr) => attr.as_ref(),
		}
	}
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
