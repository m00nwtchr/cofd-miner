use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
	dot_range::num_to_dots,
	traits::{Template, Trait},
	DOT_CHAR,
};

// #[derive(Debug, Serialize, Deserialize)]
// pub enum NumberPrereq {
// 	Equal(u8),
// 	Greater(u8),
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Prerequisite {
	Template(Template),

	Trait(Trait, u8),
	TraitOr(Vec<Trait>, u8),

	// Or(Vec<Prerequisite>),
	Not(Box<Prerequisite>),

	Unknown(String, u8),
	#[serde(untagged)]
	Special(String),
}

impl From<Template> for Prerequisite {
	fn from(value: Template) -> Self {
		Prerequisite::Template(value)
	}
}

fn parse_val(val: &str) -> Option<u8> {
	let val = val.trim_end_matches('+');
	if val.chars().all(|c| c == DOT_CHAR) {
		Some(val.chars().count() as u8)
	} else {
		val.parse().ok()
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
						parse_val(dots).unwrap_or(0),
					))
				} else {
					Trait::from_str(prereq)
						.map(|trait_| Prerequisite::Trait(trait_, parse_val(dots).unwrap_or(0)))
						.or_else(|_| {
							Ok(parse_val(dots)
								.map(|d| Prerequisite::Unknown(prereq.to_owned(), d))
								.unwrap_or_else(|| Prerequisite::Special(s.to_owned())))
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
						str += " or "
					}
					str += trait_.as_ref();
				}
				f.write_fmt(format_args!("{str} {}", num_to_dots(*num)))
			}
			Prerequisite::Not(_) => todo!(),
			Prerequisite::Unknown(str, num) => {
				f.write_fmt(format_args!("{str} {}", num_to_dots(*num)))
			}
			Prerequisite::Special(special) => f.write_str(special),
		}
	}
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum PrereqKind {
// 	Template,
// 	Attribute,
// 	Skill,
// }

pub struct Prerequisites {}
