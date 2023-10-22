use std::{collections::BTreeMap, ops::RangeFrom, path::Path};

use anyhow::Result;
use cofd_meta_schema::{MyRangeFrom, Op, PageKind, SectionMeta, SourceMeta, Span};
use cofd_schema::item::Item;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::parse::PdfExtract;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
	pub kind: PageKind,
	pub extract: String,
}

impl Section {
	pub fn parse(&self) -> Vec<Item> {
		crate::parse::parse(&self.kind, &self.extract)
	}
}

pub fn extract_section(
	pages: &BTreeMap<usize, String>,
	section: &SectionMeta,
	flag: bool,
) -> Section {
	let vec: Vec<String> = pages
		.range(section.pages.clone())
		.flat_map(|(_, page)| page.split('\n').filter(|line| !line.is_empty()))
		.map(str::to_owned)
		.collect();

	let extract = vec.join("\n");

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
						// let range = range.clone();
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
	}
}

pub fn extract_text(path: impl AsRef<Path>, source_meta: &SourceMeta) -> Result<PdfExtract> {
	let pages = crate::backend::extract_pages(path)?;
	let sections = source_meta
		.sections
		.par_iter()
		.map(|section| extract_section(&pages, section, false))
		.collect();

	Ok(PdfExtract {
		info: source_meta.info,
		sections,
	})
}
