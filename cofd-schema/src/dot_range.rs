use std::{
	ops::{RangeFrom, RangeInclusive},
	str::FromStr,
};

use derive_more::Display;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{error, DOT_CHAR};

#[derive(Serialize, Clone, Debug, Deserialize, PartialEq, Eq, Display)]
#[serde(untagged)]
pub enum DotRange {
	#[display(fmt = "{}", "num_to_dots(*_0)")]
	Num(u8),
	#[display(fmt = "{}", "_0.iter().map(|n| num_to_dots(*n)).join(\", \")")]
	Set(Vec<u8>),
	#[display(fmt = "{} to {}", "num_to_dots(*_0.start())", "num_to_dots(*_0.end())")]
	Range(RangeInclusive<u8>),
	#[display(fmt = "{}+", "num_to_dots(_0.start)")]
	RangeFrom(RangeFrom<u8>),
}

impl Default for DotRange {
	fn default() -> Self {
		Self::Num(0)
	}
}

/**
 * Function to convert a string of '•' characters to a number
 * # Errors
 * If the length of the string exceeds [`u8::MAX`], or the string contains non-'•' or whitespace characters
 */
pub fn dots_to_num(str: &str) -> Result<u8, error::ParseError> {
	let str = str.trim().trim_end_matches('+');

	if str.is_empty() {
		Ok(0)
	} else if str.chars().all(|f| f.eq(&DOT_CHAR)) {
		u8::try_from(str.chars().count()).map_err(Into::into)
	} else {
		Err(error::ParseError::BadFormat(
			"String contains non dot characters".to_owned(),
		))
	}
}

pub fn num_to_dots(n: impl Into<usize>) -> String {
	String::from(DOT_CHAR).repeat(n.into())
}

impl FromStr for DotRange {
	type Err = error::ParseError;

	fn from_str(arg: &str) -> std::result::Result<Self, Self::Err> {
		let binding = arg
			.to_lowercase()
			.replace(' ', "")
			.replace("or", ",")
			.replace("to", "-");
		let value: Vec<_> = binding
			.split(|c: char| c.eq(&',') || c.eq(&'-'))
			.filter(|str| !str.is_empty())
			.collect();

		Ok(if value.len() == 1 {
			let value = value[0];
			if value.contains('+') {
				DotRange::RangeFrom((dots_to_num(value.trim_end_matches('+'))?)..)
			} else {
				DotRange::Num(dots_to_num(value).unwrap_or(0))
			}
		} else if value.len() == 2 && binding.contains('-') {
			DotRange::Range(dots_to_num(value[0]).unwrap()..=dots_to_num(value[1]).unwrap())
		} else {
			DotRange::Set(
				value
					.iter()
					.filter_map(|str| dots_to_num(str).ok())
					.collect(),
			)
		})
	}
}
