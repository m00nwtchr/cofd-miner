use std::ops::{RangeFrom, RangeInclusive};

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

impl From<&[&str]> for DotRange {
	fn from(value: &[&str]) -> Self {
		if value.len() == 1 {
			let value = value[0];
			if value.contains('+') {
				DotRange::RangeFrom(MyRangeFrom {
					start: dot_to_num(value.trim_end_matches('+')).unwrap_or(0),
				})
			} else {
				DotRange::Num(dot_to_num(value).unwrap_or(0))
			}
		} else if value.len() == 3 && value.contains(&"to") {
			DotRange::Range(dot_to_num(value[0]).unwrap()..=dot_to_num(value[2]).unwrap())
		} else {
			DotRange::Set(value.iter().filter_map(|str| dot_to_num(str)).collect())
		}
	}
}
