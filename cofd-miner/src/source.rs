use std::{collections::HashMap, ops::Range, path::Path};

use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub use crate::backend::extract_pages;
use crate::{backend::PdfText, parse::PdfExtract};
use cofd_meta::{Op, PageKind, SectionMeta, SectionRange, SourceMeta};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
	pub kind: PageKind,
	pub extract: String,
	pub original: String,
	pub page_ranges: HashMap<usize, Range<usize>>,
}

pub fn process_section(
	pages: &PdfText,
	section: &SectionMeta,
	flag: bool,
) -> anyhow::Result<Section> {
	let pages: PdfText = pages
		.range(section.pages.clone())
		.map(|(i, p)| (*i, p.clone()))
		.collect();
	// .map(|(page_i, page)| {
	// 	(
	// 		*page_i,
	// 		page
	// 		.split('\n')
	// 			.filter(|line| !line.is_empty())
	// 			.map(str::to_owned)
	// 			.collect::<Vec<String>>()
	// 			.join("\n"),
	// 	)
	// })
	// // .map(str::to_owned)
	// .collect();

	let mut page_ranges = HashMap::new();
	let mut start = 0;
	let mut end = 0;
	for (i, page) in &pages {
		let page = page.join("\n");
		end += page.len();
		page_ranges.insert(*i, start..end);
		start = end;
	}

	let extract = pages.into_values().flatten().collect::<Vec<String>>();
	let extract = if flag {
		extract
	} else if let Some(range) = &section.range {
		match range {
			SectionRange::Range(range) => extract
				.get(range.clone())
				.map(ToOwned::to_owned)
				.unwrap_or(extract),
			SectionRange::Regex(regex) => {
				let extract_str = extract.join("\n");
				regex
					.captures(&extract_str)
					.and_then(|c| c.get(1).or_else(|| c.get(0)))
					.map(|m| m.as_str().split('\n').map(str::to_owned).collect())
					.unwrap_or(extract)
			}
		}
	} else {
		extract
	};

	let original = extract
		.join("\n")
		.replace(['‘', '’'], "'")
		.replace('–', "-");
	let mut extract = original.clone();

	if !flag {
		for op in &section.ops {
			match op {
				Op::Replace { range, replace } => {
					// extract.replace_range(range.clone(), replace);
				}
				Op::Insert { pos, char } => {
					// extract.insert(*pos, *char);
				}
				Op::Delete { range } => {
					// extract.replace_range(range.clone(), "");
				}
				Op::Move { range, pos } => {
					// let str = extract[range.clone()].to_owned();
					//
					// extract.insert_str(*pos, &str);
					//
					// if range.start() > pos {
					// 	extract.replace_range(
					// 		(range.start() + str.len())..(range.end() + str.len()),
					// 		"",
					// 	);
					// }
				}
				Op::RegexReplace { regex, replace } => {
					extract = regex.replace_all(&extract, replace).into_owned();
				}
			}
		}
	}

	Ok(Section {
		original,
		extract,
		kind: section.kind.clone(),
		page_ranges,
	})
}

pub fn extract_text(path: impl AsRef<Path>, source_meta: &SourceMeta) -> Result<PdfExtract> {
	let pages = crate::backend::extract_pages(path)?;
	let sections: Result<Vec<_>> = source_meta
		.sections
		.par_iter()
		.map(|section| process_section(&pages, section, false))
		.collect();

	Ok(PdfExtract {
		info: source_meta.info.clone(),
		sections: sections?,
	})
}
