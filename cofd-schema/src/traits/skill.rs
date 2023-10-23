use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIs, EnumString};

pub(crate) trait SkillMarker {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, AsRefStr)]
#[strum(ascii_case_insensitive)]
pub enum MentalSkill {
	Academics,
	Computer,
	Crafts,
	Investigation,
	Medicine,
	Occult,
	Politics,
	Science,

	// DE Skills
	Enigmas,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, AsRefStr)]
#[strum(ascii_case_insensitive)]
pub enum PhysicalSkill {
	Athletics,
	Brawl,
	Drive,
	Firearms,
	Larceny,
	Stealth,
	Survival,
	Weaponry,

	// DE Skills
	Archery,
	Riding,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, AsRefStr)]
#[strum(ascii_case_insensitive)]
pub enum SocialSkill {
	AnimalKen,
	Empathy,
	Expression,
	Intimidation,
	Persuasion,
	Socialize,
	Streetwise,
	Subterfuge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumIs)]
#[serde(untagged)]
pub enum Skill {
	Mental(MentalSkill),
	Physical(PhysicalSkill),
	Social(SocialSkill),
}

impl SkillMarker for MentalSkill {}
impl SkillMarker for PhysicalSkill {}
impl SkillMarker for SocialSkill {}
impl SkillMarker for Skill {}

impl FromStr for Skill {
	type Err = strum::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		MentalSkill::from_str(s)
			.map(Into::into)
			.or_else(|_| PhysicalSkill::from_str(s).map(Into::into))
			.or_else(|_| SocialSkill::from_str(s).map(Into::into))
	}
}

impl Display for Skill {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_ref())
	}
}

impl AsRef<str> for Skill {
	fn as_ref(&self) -> &str {
		match self {
			Skill::Mental(s) => s.as_ref(),
			Skill::Physical(s) => s.as_ref(),
			Skill::Social(s) => s.as_ref(),
		}
	}
}

impl From<MentalSkill> for Skill {
	fn from(value: MentalSkill) -> Self {
		Skill::Mental(value)
	}
}

impl From<PhysicalSkill> for Skill {
	fn from(value: PhysicalSkill) -> Self {
		Skill::Physical(value)
	}
}

impl From<SocialSkill> for Skill {
	fn from(value: SocialSkill) -> Self {
		Skill::Social(value)
	}
}
