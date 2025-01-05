use std::{collections::BTreeMap, path::Path, result::Result};

use anyhow::anyhow;
use mupdf::{Document, TextPageOptions};

use super::PdfText;
use crate::DOT_REGEX;

const THRESHOLD: f32 = 240.0;

pub fn extract_pages(path: impl AsRef<Path>) -> anyhow::Result<PdfText> {
	let path_str = path
		.as_ref()
		.to_str()
		.ok_or_else(|| anyhow!("Path is not a valid UTF-8 string"))?;
	let document = Document::open(path_str)?;

	let mut pages = BTreeMap::new();

	for (page_index, text_page) in document
		.pages()?
		.filter_map(Result::ok)
		.filter_map(|p| p.to_text_page(TextPageOptions::empty()).ok())
		.enumerate()
	{
		let page_lines = process_page(&text_page);
		pages.insert(page_index, page_lines);
	}

	Ok(pages)
}

fn process_page(text_page: &mupdf::TextPage) -> Vec<String> {
	let mut left_indent = (f32::MAX, f32::MIN);
	let mut right_indent = (f32::MAX, f32::MIN);

	let mut last_y = 0.0;
	let mut blank = false;

	let mut lines = Vec::new();

	for block in text_page.blocks() {
		for line in block.lines() {
			let x = line.bounds().x0;
			let y = line.bounds().y0;
			let line_text = line.chars().filter_map(|c| c.char()).collect::<String>();

			let y_shift = (y - last_y).floor();
			last_y = y;

			if y_shift.abs() > 100.0 {
				blank = y_shift > 0.0;
			}

			if blank || line_text.trim().chars().all(char::is_numeric) {
				continue;
			}

			if x > THRESHOLD {
				left_indent.0 = left_indent.0.min(x);
			} else {
				right_indent.0 = right_indent.0.min(x);
			}

			lines.push((x, line_text));
		}
	}

	format_lines(lines, left_indent.0, right_indent.0)
}

fn format_lines(lines: Vec<(f32, String)>, left_bound: f32, right_bound: f32) -> Vec<String> {
	let mut last_indent = 0.0;
	let mut last_has_dot = false;

	lines
		.into_iter()
		.map(|(x, line)| {
			let is_right = x > THRESHOLD;
			let min_indent = if is_right { right_bound } else { left_bound };
			let indent = (x - min_indent).floor();
			let has_dot = DOT_REGEX.is_match(&line);

			let should_tab = if indent > last_indent {
				!last_has_dot || has_dot
			} else if indent < last_indent {
				has_dot
			} else {
				false
			};

			last_indent = indent;
			last_has_dot = has_dot;

			let prefix = if should_tab { "\t" } else { "" };
			format!("{prefix}{line}")
		})
		.collect()
}
