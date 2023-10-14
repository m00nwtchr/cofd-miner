use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
	traits::{attribute::Attribute, skill::Skill, SupernaturalTolerance, Template, Trait},
	DOT_CHAR,
};

// #[derive(Debug, Serialize, Deserialize)]
// pub enum NumberPrereq {
// 	Equal(u8),
// 	Greater(u8),
// }

#[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
pub enum Prerequisite {
	Template(Template),
	Attribute(Attribute, u8),
	Skill(Skill, u8),
	Trait(Trait, u8),
	SupernaturalTolerance(SupernaturalTolerance, u8),
	Or(Vec<Prerequisite>),
	Not(Box<Prerequisite>),
	Unknown(String, u8),
	#[serde(untagged)]
	String(String), // Any(PrereqKind),
}

impl From<Template> for Prerequisite {
	fn from(value: Template) -> Self {
		Prerequisite::Template(value)
	}
}

fn parse_val(val: &str) -> Option<u8> {
	let val = val.trim_end_matches("+");
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
			if let Some((prereq, dots)) = s.rsplit_once(' ') {
				Attribute::from_str(prereq)
					.map(|attr| Prerequisite::Attribute(attr, parse_val(dots).unwrap_or(0)))
					.or_else(|_| {
						Skill::from_str(prereq)
							.map(|skill| Prerequisite::Skill(skill, parse_val(dots).unwrap_or(0)))
					})
					.or_else(|_| {
						SupernaturalTolerance::from_str(prereq).map(|st| {
							Prerequisite::SupernaturalTolerance(st, parse_val(dots).unwrap_or(0))
						})
					})
					.or_else(|_| {
						Trait::from_str(prereq)
							.map(|trait_| Prerequisite::Trait(trait_, parse_val(dots).unwrap_or(0)))
					})
					.or_else(|_| {
						Ok(parse_val(dots)
							.map(|d| Prerequisite::Unknown(prereq.to_owned(), d))
							.unwrap_or_else(|| Prerequisite::String(s.to_owned())))
					})
			} else {
				Ok(Prerequisite::String(s.to_owned()))
			}
		})
	}
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum PrereqKind {
// 	Template,
// 	Attribute,
// 	Skill,
// }

pub struct Prerequisites {}
