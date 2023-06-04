use std::ops::Range;

use crate::{page_kind::PageKind, PageIndex};

pub struct PageSpanDef {
	pub range: Range<PageIndex>,
	pub kind: PageKind,
}

pub struct SourceFileDef {
	pub hash: u64,
	pub spans: Vec<PageSpanDef>,
}

impl SourceFileDef {}
