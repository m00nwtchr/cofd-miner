use std::{ops::Deref, str::FromStr};

use derive_more::Display;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
	dot_range::{dots_to_num, num_to_dots},
	error,
	traits::{Template, Trait},
};

/**
 * Level-rated prerequisite types
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[serde(untagged)]
pub enum RatedPrerequisiteKey {
	Trait(Trait),
	Unknown(String),
}

impl FromStr for RatedPrerequisiteKey {
	type Err = error::ParseError;

	fn from_str(prereq: &str) -> Result<Self, Self::Err> {
		Trait::from_str(prereq)
			.map(Self::Trait)
			.or_else(|_| Ok(Self::Unknown(prereq.to_owned())))
	}
}

/**
 * Prerequisites with level ratings
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display(fmt = "{_0} {}", "num_to_dots(*_1)")]
pub struct RatedPrerequisite(RatedPrerequisiteKey, u8);

impl FromStr for RatedPrerequisite {
	type Err = error::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if let Some((prereq, dots)) = s.find(" •").map(|p| s.split_at(p)) {
			dots_to_num(dots).and_then(|dots| {
				RatedPrerequisiteKey::from_str(prereq).map(|prereq| RatedPrerequisite(prereq, dots))
			})
		} else {
			Err(error::ParseError::BadFormat(
				"String is not in the format: {key} {dots}".to_owned(),
			))
		}
	}
}

/**
 * All prerequisite types
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[serde(untagged)]
pub enum PrerequisiteKey {
	Template(Template),
	Rated(RatedPrerequisite),
}

impl FromStr for PrerequisiteKey {
	type Err = error::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Template::from_str(s)
			.map(Self::Template)
			.or_else(|_| RatedPrerequisite::from_str(s).map(Self::Rated))
		// .or_else(|_| Ok(Self::Unknown(s.to_owned())))
	}
}

/**
 * A single prerequisite, or a set of OR prerequisites
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[serde(untagged)]
pub enum Prerequisite {
	Key(PrerequisiteKey),
	#[display(fmt = "{}", "_0.iter().join(\" or \")")]
	Or(Vec<PrerequisiteKey>),
	Unknown(String),
}

impl FromStr for Prerequisite {
	type Err = error::ParseError;

	fn from_str(prereq: &str) -> Result<Self, Self::Err> {
		let prereq = prereq.trim();
		PrerequisiteKey::from_str(prereq)
			.map(Self::Key)
			.or_else(|_| {
				if let Some((l, r)) = prereq.split_once(" or ") {
					let l = PrerequisiteKey::from_str(l.trim())?;
					let r = PrerequisiteKey::from_str(r.trim())?;

					Ok(Self::Or(vec![l, r]))
				} else {
					Ok(Self::Unknown(prereq.to_owned()))
				}
			})
	}
}

/**
 * A set of prerequisites (AND)
 */
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[serde(transparent)]
#[display(fmt = "{}", "_0.iter().join(\", \")")]
pub struct Prerequisites(Vec<Prerequisite>);

impl From<Vec<Prerequisite>> for Prerequisites {
	fn from(value: Vec<Prerequisite>) -> Self {
		Prerequisites(value)
	}
}

impl Prerequisites {
	#[must_use]
	pub fn unwrap(self) -> Vec<Prerequisite> {
		self.0
	}
}

impl Deref for Prerequisites {
	type Target = Vec<Prerequisite>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl FromStr for Prerequisites {
	type Err = error::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let ok: Result<Vec<_>, _> = s
			.split(", ")
			.filter(|f| !f.is_empty())
			.map(FromStr::from_str)
			.collect();

		Ok(Prerequisites(ok?))
	}
}
