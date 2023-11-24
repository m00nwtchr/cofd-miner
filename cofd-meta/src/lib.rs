use std::ops::{Range, RangeInclusive};

use serde::{Deserialize, Serialize};

pub use crate::page_kind::PageKind;
use cofd_schema::book::BookInfo;

mod page_kind;

fn unnamed() -> String {
	"Unnamed".to_owned()
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionMeta {
	#[serde(default = "unnamed", skip_serializing_if = "String::is_empty")]
	pub name: String,
	pub pages: RangeInclusive<usize>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub range: Option<Range<usize>>,
	pub kind: PageKind,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ops: Vec<Op>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SourceMeta {
	pub info: BookInfo,
	pub sections: Vec<SectionMeta>,
}

impl SourceMeta {}
