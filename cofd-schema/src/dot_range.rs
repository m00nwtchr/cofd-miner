use std::{
	ops::{RangeFrom, RangeInclusive},
	str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct MyRangeFrom {
	start: u8,
}
impl From<MyRangeFrom> for RangeFrom<u8> {
	fn from(val: MyRangeFrom) -> Self {
		val.start..
	}
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(untagged)]
pub enum DotRange {
	Num(u8),
	Set(Vec<u8>),
	Range(RangeInclusive<u8>),
	RangeFrom(MyRangeFrom),
}

fn dot_to_num(str: &str) -> Option<u8> {
	if str.chars().all(|f| f.eq(&'•')) {
		Some(str.chars().count() as u8)
	} else {
		None
	}
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
					start: dot_to_num(value.trim_end_matches('+')).unwrap_or(0),
				})
			} else {
				DotRange::Num(dot_to_num(value).unwrap_or(0))
			}
		} else if value.len() == 2 && binding.contains('-') {
			DotRange::Range(dot_to_num(value[0]).unwrap()..=dot_to_num(value[1]).unwrap())
		} else {
			DotRange::Set(value.iter().filter_map(|str| dot_to_num(str)).collect())
		})
	}
}
