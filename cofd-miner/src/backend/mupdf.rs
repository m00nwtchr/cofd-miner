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
	let mut pages = BTreeMap::new();

	for (i, text_page) in document
		.pages()?
		.filter_map(Result::ok)
		.filter_map(|p| p.to_text_page(TextPageOptions::empty()).ok())
		.enumerate()
	{
		let mut l_indent = (f32::MAX, f32::MIN);
		let mut r_indent = (f32::MAX, f32::MIN);

		let mut last_y = 0.0;
		let mut blank = false;

		let mut lines = Vec::new();

		for block in text_page.blocks() {
			for line in block.lines() {
				let x = line.bounds().x0;
				let y = line.bounds().y0;
				let line = line.chars().filter_map(|c| c.char()).collect::<String>();

				let y_shift = f32::floor(y - last_y);

				if y_shift > 100.0 {
					blank = true; // End of Page Content
				} else if y_shift < -100.0 {
					blank = false; // New Page
				}
				last_y = y;

				if blank || line.trim().chars().all(char::is_numeric) {
					// Some(format!(
					// 	"{min_x}{indent}:BLANK:{}",
					// 	l.chars().filter_map(|c| c.char()).collect::<String>()
					// ));
					continue;
				}

				if x > THRESHOLD {
					if x < r_indent.0 {
						r_indent.0 = x;
					}
				} else if x < l_indent.0 {
					l_indent.0 = x;
				}

				lines.push((x, line));
			}
		}

		let mut last_x = 0.0;
		let mut last_indent = 0.0;

		// let mut last_has_dot = false;
		// let mut dot_line_indent = f32::MAX;
		// let mut dot_paragraph_indent = f32::MAX;
		// let mut pre_dot_indent = f32::MAX;

		let mut last_tab = false;

		let lines = lines
			.into_iter()
			.map(|(x, line)| {
				let min_x = if x < THRESHOLD {
					l_indent.0
				} else {
					r_indent.0
				};
				let indent = f32::floor(x - min_x);

				//
				// let x = f32::floor(l.bounds().x0);
				// let y = f32::floor(l.bounds().y0);
				//

				#[allow(clippy::if_same_then_else, clippy::nonminimal_bool)]
				let tab = if indent > last_indent {
					// if indent == 0.0 {
					// 	// Jump to other column
					// 	false
					// } else {
					// 	true
					// }
					true
				} else if indent < last_indent {
					// line.insert_str(0, "LESS:");
					false
				} else {
					// line.insert_str(0, "LAST:");
					last_tab
				};

				// if last_has_dot && !dot && dot_paragraph_indent == f32::MAX {
				// 	dot_paragraph_indent = indent;
				// }

				last_x = x;

				last_indent = indent;
				last_tab = tab;
				// last_has_dot = dot;

				let tab = if tab { "\t" } else { "" };
				// let dot = if dot { "\t" } else { "" };
				format!("{tab}{line}")
			})
			.collect();

		pages.insert(i, lines);
	}

	Ok(pages)
}
