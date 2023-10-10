use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Serialize, Deserialize, EnumString)]
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
}

#[derive(Debug, Serialize, Deserialize, EnumString)]
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
}

#[derive(Debug, Serialize, Deserialize, EnumString)]
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Skill {
	Mental(MentalSkill),
	Physical(PhysicalSkill),
	Social(SocialSkill),
}

impl FromStr for Skill {
	type Err = strum::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		MentalSkill::from_str(s)
			.map(Into::into)
			.or_else(|_| PhysicalSkill::from_str(s).map(Into::into))
			.or_else(|_| SocialSkill::from_str(s).map(Into::into))
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
