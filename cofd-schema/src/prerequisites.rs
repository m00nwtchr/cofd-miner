use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
	dot_range::{dots_to_num, num_to_dots},
	error,
	traits::{Template, Trait},
};

/**
 * Level-rated prerequisite types
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RatedPrerequisiteKey {
	Trait(Trait),
	Unknown(String),
}

impl Display for RatedPrerequisiteKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Trait(trait_) => trait_.fmt(f),
			Self::Unknown(str) => str.fmt(f),
		}
	}
}

impl FromStr for RatedPrerequisiteKey {
	type Err = error::ParseError;

	fn from_str(prereq: &str) -> Result<Self, Self::Err> {
		Trait::from_str(prereq)
			.map(Self::Trait)
			.or_else(|_| Ok(Self::Unknown(prereq.to_string())))
	}
}

/**
 * Prerequisites with level ratings
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatedPrerequisite(RatedPrerequisiteKey, u8);

impl Display for RatedPrerequisite {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {}", self.0, num_to_dots(self.1))
	}
}

impl FromStr for RatedPrerequisite {
	type Err = error::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if let Some((prereq, dots)) = s.find(" •").map(|p| s.split_at(p)) {
			dots_to_num(dots).and_then(|dots| {
				RatedPrerequisiteKey::from_str(prereq).map(|prereq| RatedPrerequisite(prereq, dots))
			})
		} else {
			Err(error::ParseError::BadFormat(
				"String is not in the format: {key} {dots}".to_string(),
			))
		}
	}
}

/**
 * All prerequisite types
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrerequisiteKey {
	Template(Template),
	Rated(RatedPrerequisite),
}

impl Display for PrerequisiteKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PrerequisiteKey::Template(t) => t.fmt(f),
			PrerequisiteKey::Rated(r) => r.fmt(f),
			// PrerequisiteKey::Unknown(s) => s.fmt(f),
		}
	}
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Prerequisite {
	Key(PrerequisiteKey),
	Or(Vec<PrerequisiteKey>),
	Unknown(String),
}

impl Display for Prerequisite {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Prerequisite::Key(k) => k.fmt(f),
			Prerequisite::Or(prereqs) => {
				let mut out = String::new();
				for prereq in &prereqs[0..prereqs.len() - 1] {
					out.push_str(&prereq.to_string());
					out.push_str(" or ");
				}
				out.push_str(&prereqs[prereqs.len() - 1].to_string());
				write!(f, "{out}")
			}
			Prerequisite::Unknown(s) => s.fmt(f),
		}
	}
}

impl FromStr for Prerequisite {
	type Err = error::ParseError;

	fn from_str(prereq: &str) -> Result<Self, Self::Err> {
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
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
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

impl Display for Prerequisites {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.iter()
			.try_fold((), |_result, prereq| write!(f, "{prereq}, "))
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
