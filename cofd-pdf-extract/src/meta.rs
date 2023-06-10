use std::ops::{Range, RangeFrom, RangeInclusive};

use serde::{Deserialize, Serialize};

use crate::page_kind::PageKind;

fn is_none<T>(v: &Option<T>) -> bool {
	v.is_none()
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Span {
	Range(Range<usize>),
	From(MyRangeFrom),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectionDefinition {
	pub pages: RangeInclusive<usize>,
	#[serde(default, skip_serializing_if = "is_none")]
	pub range: Option<Span>,
	pub kind: PageKind,
}

#[derive(Serialize, Deserialize)]
pub struct SourceMeta {
	pub hash: u64,
	pub timestamp: u32,
	pub sections: Vec<SectionDefinition>,
}

impl SourceMeta {}
