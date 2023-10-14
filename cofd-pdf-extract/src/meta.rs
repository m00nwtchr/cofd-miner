use std::ops::{Range, RangeFrom, RangeInclusive};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use cofd_schema::book::BookInfo;

use crate::page_kind::PageKind;

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
	Replace {
		range: RangeInclusive<usize>,
		replace: String,
	},
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

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDefinition {
	#[serde(default = "unnamed", skip_serializing_if = "is_empty_str")]
	pub name: String,
	pub pages: RangeInclusive<usize>,
	#[serde(default)]
	pub range: Option<Span>,
	pub kind: PageKind,
	#[serde(default, skip_serializing_if = "is_empty")]
	pub ops: Vec<Op>,
}

#[derive(Serialize, Deserialize)]
pub struct SourceMeta {
	pub info: BookInfo,
	pub sections: Vec<SectionDefinition>,
}

impl SourceMeta {}
