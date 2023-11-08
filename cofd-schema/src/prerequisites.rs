use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
	dot_range::{dots_to_num, num_to_dots},
	traits::{Template, Trait},
};

// #[derive(Debug, Serialize, Deserialize)]
// pub enum NumberPrereq {
// 	Equal(u8),
// 	Greater(u8),
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Prerequisite {
	Template(Template),

	Trait(Trait, u8),
	TraitOr(Vec<Trait>, u8),

	// Or(Vec<Prerequisite>),
	// Not(Box<Prerequisite>),
	Unknown(String, u8),
	#[serde(untagged)]
	Special(String),
}

impl From<Template> for Prerequisite {
	fn from(value: Template) -> Self {
		Prerequisite::Template(value)
	}
}

impl FromStr for Prerequisite {
	type Err = strum::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Template::from_str(s).map(Into::into).or_else(|_| {
			if let Some((prereq, dots)) = s.find('•').map(|f| s.split_at(f - 1)) {
				let prereq = prereq.trim();
				let dots = dots.trim();
				if let Some((l, r)) = prereq.split_once(" or ") {
					let l = Trait::from_str(l.trim())?;
					let r = Trait::from_str(r.trim())?;

					Ok(Prerequisite::TraitOr(
						vec![l, r],
						dots_to_num(dots).unwrap_or(0),
					))
				} else {
					Trait::from_str(prereq)
						.map(|trait_| Prerequisite::Trait(trait_, dots_to_num(dots).unwrap_or(0)))
						.or_else(|_| {
							Ok(dots_to_num(dots).map_or_else(
								|| Prerequisite::Special(s.to_owned()),
								|d| Prerequisite::Unknown(prereq.to_owned(), d),
							))
						})
				}
			} else {
				Ok(Prerequisite::Special(s.to_owned()))
			}
		})
	}
}

impl Display for Prerequisite {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Prerequisite::Template(template) => template.fmt(f),
			Prerequisite::Trait(trait_, num) => {
				f.write_fmt(format_args!("{trait_} {}", num_to_dots(*num)))
			}
			Prerequisite::TraitOr(traits, num) => {
				let mut str = String::new();
				for trait_ in traits {
					if !str.is_empty() {
						str += " or ";
					}
					str += trait_.as_ref();
				}
				f.write_fmt(format_args!("{str} {}", num_to_dots(*num)))
			}
			// Prerequisite::Not(_) => todo!(),
			Prerequisite::Unknown(str, num) => {
				f.write_fmt(format_args!("{str} {}", num_to_dots(*num)))
			}
			Prerequisite::Special(special) => f.write_str(special),
		}
	}
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Prerequisites(Vec<Prerequisite>);

impl From<Vec<Prerequisite>> for Prerequisites {
	fn from(value: Vec<Prerequisite>) -> Self {
		Prerequisites(value)
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
		let mut str = String::new();
		for prereq in &self.0 {
			if !str.is_empty() {
				str += ", ";
			}
			str += &prereq.to_string();
		}
		f.write_str(&str)
	}
}

impl FromStr for Prerequisites {
	type Err = strum::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let ok: Result<Vec<_>, _> = s
			.split(", ")
			.filter(|f| !f.is_empty())
			.map(FromStr::from_str)
			.collect();

		Ok(Prerequisites(ok?))
	}
}
