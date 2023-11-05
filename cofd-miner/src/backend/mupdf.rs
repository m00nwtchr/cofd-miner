use std::{collections::BTreeMap, path::Path};

use anyhow::anyhow;
use mupdf::Document;

pub fn extract_pages(path: impl AsRef<Path>) -> anyhow::Result<BTreeMap<usize, String>> {
	let document = Document::open(
		path.as_ref()
			.to_str()
			.ok_or(anyhow!("Path is not valid utf-8 string"))?,
	)?;
	let pages = document
		.pages()?
		.enumerate()
		.filter_map(|(i, p)| p.ok().map(|p| (i, p)))
		.filter_map(|(i, page)| page.to_text().ok().map(|p| (i, p)))
		.collect();

	Ok(pages)
}
