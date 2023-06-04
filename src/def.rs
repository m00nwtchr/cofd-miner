use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::{page_kind::PageKind, PageIndex};

#[derive(Serialize, Deserialize)]
pub struct PageSpanDef {
	pub range: Range<PageIndex>,
	pub kind: PageKind,
}

#[derive(Serialize, Deserialize)]
pub struct SourceFileDef {
	pub hash: u64,
	pub timestamp: u32,
	pub spans: Vec<PageSpanDef>,
}

impl SourceFileDef {}
