use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::{
	splat::mage::{Arcanum, Practice},
	traits::skill::Skill,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, PartialEq, Eq)]
pub enum PrimaryFactor {
	Duration,
	Potency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Factor {
	Scale,
	#[serde(untagged)]
	PrimaryFactor(PrimaryFactor),
}

impl From<PrimaryFactor> for Factor {
	fn from(value: PrimaryFactor) -> Self {
		Self::PrimaryFactor(value)
	}
}

impl FromStr for Factor {
	type Err = strum::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		PrimaryFactor::from_str(s).map(Into::into).or_else(|err| {
			if s.eq_ignore_ascii_case("Scale") {
				Ok(Factor::Scale)
			} else {
				Err(err)
			}
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReachEffect {
	cost: u8,
	description: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArcanumEffectKind {
	Add,
	Substitute,

	Or(Vec<ArcanumEffectKind>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArcanumEffect {
	arcanum: Arcanum,
	rating: u8,
	description: Vec<String>,
	kind: ArcanumEffectKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Spell {
	arcana: Vec<(Arcanum, u8)>,
	practice: Practice,
	primary_factor: PrimaryFactor,
	cost: String,
	suggested_rote_skills: [Skill; 3],
	reaches: Vec<ReachEffect>,
	arcana_effects: Vec<ArcanumEffect>,
}
