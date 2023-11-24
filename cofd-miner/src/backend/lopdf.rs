use std::{collections::BTreeMap, path::Path};

use anyhow::anyhow;
use lopdf::Document;
use rayon::prelude::*;

pub fn extract_pages(path: impl AsRef<Path>) -> anyhow::Result<BTreeMap<usize, String>> {
	let document = Document::load(path)?;

	let pages = document
		.get_pages()
		.into_par_iter()
		.map(
			|(page_num, page_id): (u32, (u32, u16))| -> anyhow::Result<(u32, Vec<String>)> {
				let text = document.extract_text(&[page_num]).map_err(|e| {
					anyhow!("Failed to extract text from page {page_num} id={page_id:?}: {e:?}")
				})?;
				Ok((
					page_num,
					text.split('\n')
						.map(|s| s.trim_end().to_owned())
						.collect::<Vec<String>>(),
				))
			},
		)
		.collect();
}
