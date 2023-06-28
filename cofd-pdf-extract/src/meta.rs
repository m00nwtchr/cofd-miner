use std::ops::{Range, RangeFrom, RangeInclusive};

use serde::{Deserialize, Serialize};

use crate::page_kind::PageKind;

fn is_none<T>(v: &Option<T>) -> bool {
	v.is_none()
}

fn is_empty<T>(v: &Vec<T>) -> bool {
	v.is_empty()
}

fn is_empty_str(str: &String) -> bool {
	str.is_empty() || str.eq("Unnamed")
}

fn unnamed() -> String {
	"Unnamed".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyRangeFrom {
	pub start: usize,
}
impl Into<RangeFrom<usize>> for MyRangeFrom {
	fn into(self) -> RangeFrom<usize> {
		self.start..
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Span {
	Range(Range<usize>),
	From(MyRangeFrom),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Op {
	Insert {
		pos: usize,
		char: char,
	},
	Delete {
		range: RangeInclusive<usize>,
	},
	Move {
		range: RangeInclusive<usize>,
		pos: usize,
	},
	RegexReplace {
		regex: String,
		replace: String,
	},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDefinition {
	#[serde(default = "unnamed", skip_serializing_if = "is_empty_str")]
	pub name: String,
	pub pages: RangeInclusive<usize>,
	#[serde(default, skip_serializing_if = "is_none")]
	pub range: Option<Span>,
	pub kind: PageKind,
	#[serde(default, skip_serializing_if = "is_empty")]
	pub ops: Vec<Op>,
}

#[derive(Serialize, Deserialize)]
pub struct SourceMeta {
	#[serde(with = "hex")]
	pub hash: u64,
	pub timestamp: u32,
	pub sections: Vec<SectionDefinition>,
}

impl SourceMeta {}

mod hex {
	use serde::{Deserialize, Serialize};

	pub fn serialize<S>(v: &u64, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		if serializer.is_human_readable() {
			format!("{v:X}").serialize(serializer)
		} else {
			v.serialize(serializer)
		}
	}
	pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		if deserializer.is_human_readable() {
			String::deserialize(deserializer)
				.and_then(|str| Ok(u64::from_str_radix(&str, 16).unwrap()))
		} else {
			u64::deserialize(deserializer)
		}
	}
}
