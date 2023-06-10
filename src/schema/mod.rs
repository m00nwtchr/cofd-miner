use std::ops::{Range, RangeFrom, RangeInclusive};

use serde::{Deserialize, Serialize};

mod prerequisites;

#[derive(Serialize, Deserialize)]
pub struct MyRangeFrom {
	start: u8,
}
impl Into<RangeFrom<u8>> for MyRangeFrom {
	fn into(self) -> RangeFrom<u8> {
		self.start..
	}
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DotRange {
	Num(u8),
	Set(Vec<u8>),
	Range(RangeInclusive<u8>),
	RangeFrom(MyRangeFrom),
}

// enum Token {
// 	Num(i8),
// 	To,
// }

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
					start: dot_to_num(&value.trim_end_matches('+')).unwrap_or(0),
				})
			} else {
				DotRange::Num(dot_to_num(&value).unwrap_or(0))
			}
		} else if value.len() == 3 && value.contains(&"to") {
			DotRange::Range(dot_to_num(&value[0]).unwrap()..=dot_to_num(&value[2]).unwrap())
		} else {
			DotRange::Set(value.iter().filter_map(|str| dot_to_num(str)).collect())
		}
	}
}

impl DotRange {}
