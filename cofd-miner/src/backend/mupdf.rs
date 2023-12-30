use std::{collections::BTreeMap, path::Path, result::Result};

use anyhow::anyhow;
use mupdf::{Document, TextPageOptions};

use super::PdfText;

const THRESHOLD: f32 = 240.0;

pub fn extract_pages(path: impl AsRef<Path>) -> anyhow::Result<PdfText> {
	let document = Document::open(
		path.as_ref()
			.to_str()
			.ok_or(anyhow!("Path is not valid utf-8 string"))?,
	)?;
	//
	// let pages: BTreeMap<_, _> = document
	// 	.pages()?
	// 	.enumerate()
	// 	.filter_map(|(i, p)| p.ok().map(|p| (i, p)))
	// 	.filter_map(|(i, page)| page.to_text().ok().map(|p| (i, p)))
	// 	.collect();

	let mut pages = BTreeMap::new();

	for (i, text_page) in document
		.pages()?
		.filter_map(Result::ok)
		.filter_map(|p| p.to_text_page(TextPageOptions::empty()).ok())
		.enumerate()
	{
		let mut l_indent = (f32::MAX, f32::MIN);
		let mut r_indent = (f32::MAX, f32::MIN);

		for block in text_page.blocks() {
			for line in block.lines() {
				let indent = line.bounds().x0;

				if indent > THRESHOLD {
					if indent < r_indent.0 {
						r_indent.0 = indent;
					}
				} else if indent < l_indent.0 {
					l_indent.0 = indent;
				}
			}
		}

		let mut lines = Vec::new();
		// let mut last_y = 0.0;

		let mut last_x = 0.0;
		let mut last_indent = false;

		for block in text_page.blocks() {
			let block: Vec<_> = block
				.lines()
				.map(|l| {
					let x = l.bounds().x0;
					let min_x = if x < THRESHOLD {
						l_indent.0
					} else {
						r_indent.0
					};
					let indent = f32::floor(x - min_x);
					//
					let x = f32::floor(l.bounds().x0);
					let y = f32::floor(l.bounds().y0);
					// //
					// // let c = f32::abs(y - last_y);
					// // last_y = y;

					let indent = if x > last_x {
						if indent == 0.0 { // Jump to other column
							false
						} else {
							true
						}
					} else if x < last_x {
						false
					} else {
						last_indent
					};

					last_x = x;
					last_indent = indent;

					format!(
						"{}{}",
						if indent { "\t" } else { "" },
						l.chars().filter_map(|c| c.char()).collect::<String>()
					)
				})
				.collect();
			lines.extend(block);
		}
		pages.insert(i, lines);
	}

	Ok(pages)
}
