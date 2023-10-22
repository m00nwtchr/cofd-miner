use std::{
	collections::{BTreeMap, HashMap},
	ops::{Range, RangeFrom},
	path::Path,
};

use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub use crate::backend::extract_pages;
use crate::parse::PdfExtract;
use cofd_meta_schema::{MyRangeFrom, Op, PageKind, SectionMeta, SourceMeta, Span};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
	pub kind: PageKind,
	pub extract: String,
	pub page_ranges: HashMap<usize, Range<usize>>,
}

pub fn process_section(
	pages: &BTreeMap<usize, String>,
	section: &SectionMeta,
	flag: bool,
) -> Section {
	let pages: BTreeMap<usize, String> = pages
		.range(section.pages.clone())
		.map(|(page_i, page)| {
			(
				*page_i,
				page.split('\n')
					.filter(|line| !line.is_empty())
					.map(str::to_owned)
					.collect::<Vec<String>>()
					.join("\n"),
			)
		})
		// .map(str::to_owned)
		.collect();

	let mut page_ranges = HashMap::new();
	let mut start = 0;
	let mut end = 0;
	for (i, page) in &pages {
		end += page.len();
		page_ranges.insert(*i, start..end);
		start = end;
	}

	let extract = pages.into_values().collect::<Vec<String>>().join("\n");

	let mut extract = if flag {
		extract
	} else if let Some(range) = &section.range {
		match range {
			Span::Range(range) => extract[range.clone()].to_owned(),
			Span::From(range) => {
				extract[<MyRangeFrom as Into<RangeFrom<usize>>>::into(range.clone())].to_owned()
			}
		}
	} else {
		extract
	};

	if !flag {
		for op in &section.ops {
			match op {
				Op::Replace { range, replace } => {
					extract.replace_range(range.clone(), replace);
				}
				Op::Insert { pos, char } => {
					extract.insert(*pos, *char);
				}
				Op::Delete { range } => {
					extract.replace_range(range.clone(), "");
				}
				Op::Move { range, pos } => {
					let str = extract[range.clone()].to_owned();

					extract.insert_str(*pos, &str);

					if range.start() > pos {
						extract.replace_range(
							(range.start() + str.len())..(range.end() + str.len()),
							"",
						);
					}
				}
				Op::RegexReplace { regex, replace } => {
					let regex = Regex::new(regex).unwrap();

					extract = regex.replace_all(&extract, replace).to_string();
				} // Op::Swap { a, b } => {
				  // 	let a = a.clone();
				  // 	let astr = extract[a.clone()].to_owned();
				  // 	let bstr = extract[b.clone()].to_owned();
				  // 	println!("{a:?}, {astr}");

				  // 	extract.replace_range(a, &bstr);
				  // 	let new_bstart = extract.find(&bstr).unwrap();
				  // 	extract.replace_range(new_bstart..(new_bstart + bstr.len()), &astr);
				  // }
			}
		}
	}

	Section {
		extract,
		kind: section.kind.clone(),
		page_ranges,
	}
}

pub fn extract_text(path: impl AsRef<Path>, source_meta: &SourceMeta) -> Result<PdfExtract> {
	let pages = crate::backend::extract_pages(path)?;
	let sections = source_meta
		.sections
		.par_iter()
		.map(|section| process_section(&pages, section, false))
		.collect();

	Ok(PdfExtract {
		info: source_meta.info.clone(),
		sections,
	})
}
