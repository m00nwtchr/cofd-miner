use std::{
	fmt::Display,
	ops::{RangeFrom, RangeInclusive},
	str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::DOT_CHAR;

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct MyRangeFrom {
	pub start: u8,
}
impl From<MyRangeFrom> for RangeFrom<u8> {
	fn from(val: MyRangeFrom) -> Self {
		val.start..
	}
}

#[derive(Serialize, Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum DotRange {
	Num(u8),
	Set(Vec<u8>),
	Range(RangeInclusive<u8>),
	RangeFrom(MyRangeFrom),
}

#[must_use]
pub fn dots_to_num(str: &str) -> Option<u8> {
	let str = str.trim_end_matches('+');
	if str.chars().all(|f| f.eq(&DOT_CHAR)) {
		u8::try_from(str.chars().count()).ok()
	} else {
		None
	}
}

pub fn num_to_dots(n: impl Into<usize>) -> String {
	String::from(DOT_CHAR).repeat(n.into())
}

impl FromStr for DotRange {
	type Err = strum::ParseError;

	fn from_str(arg: &str) -> std::result::Result<Self, strum::ParseError> {
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
				DotRange::RangeFrom(MyRangeFrom {
					start: dots_to_num(value.trim_end_matches('+'))
						.ok_or(strum::ParseError::VariantNotFound)?, // TODO: custom error types
				})
			} else {
				DotRange::Num(dots_to_num(value).unwrap_or(0))
			}
		} else if value.len() == 2 && binding.contains('-') {
			DotRange::Range(dots_to_num(value[0]).unwrap()..=dots_to_num(value[1]).unwrap())
		} else {
			DotRange::Set(value.iter().filter_map(|str| dots_to_num(str)).collect())
		})
	}
}

impl Display for DotRange {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DotRange::Num(num) => f.write_str(&num_to_dots(*num)),
			DotRange::Set(set) => {
				let mut out = String::new();
				for num in set {
					if !out.is_empty() {
						out += ", ";
					}
					out += &num_to_dots(*num);
				}
				f.write_str(&out)
			}
			DotRange::Range(range) => f.write_fmt(format_args!(
				"{} to {}",
				&num_to_dots(*range.start()),
				&num_to_dots(*range.end())
			)),
			DotRange::RangeFrom(range) => {
				f.write_fmt(format_args!("{}+", &num_to_dots(range.start)))
			}
		}
	}
}
