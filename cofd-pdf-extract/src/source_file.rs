use std::{collections::HashMap, ops::RangeFrom, path::Path};

use anyhow::Result;
use mupdf::Document;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
	meta::{MyRangeFrom, SectionDefinition, SourceMeta},
	page_kind::{Item, PageKind},
};

pub fn extract_pages(path: &impl AsRef<Path>) -> Result<HashMap<usize, String>> {
	let document = Document::open(path.as_ref().to_str().unwrap())?;

	let pages = document
		.pages()?
		.enumerate()
		.filter_map(|(i, p)| p.ok().map(|p| (i, p)))
		.filter_map(|(i, page)| page.to_text().ok().map(|p| (i, p)))
		.collect();

	Ok(pages)
}

pub fn make_section(
	pages: &HashMap<usize, String>,
	section: &SectionDefinition,
	flag: bool,
) -> Section {
	let vec: Vec<String> = (section.pages.clone())
		.into_par_iter()
		.filter_map(|i| pages.get(&i))
		.flat_map(|page| {
			page.split("\n")
				.filter(|line| !line.is_empty())
				.collect::<Vec<_>>()
		})
		.map(str::to_owned)
		.collect();

	let extract = vec.join("\n");

	let mut extract = if flag {
		extract
	} else if let Some(range) = &section.range {
		match range {
			crate::meta::Span::Range(range) => extract[range.clone()].to_owned(),
			crate::meta::Span::From(range) => {
				extract[<MyRangeFrom as Into<RangeFrom<usize>>>::into(range.clone())].to_owned()
			}
		}
	} else {
		extract
	};

	if !flag {
		for op in &section.ops {
			match op {
				crate::meta::Op::Insert { pos, char } => {
					extract.insert(*pos, *char);
				}
				crate::meta::Op::Delete { range } => {
					extract.replace_range(range.clone(), "");
				}
				crate::meta::Op::Move { range, pos } => {
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
				crate::meta::Op::RegexReplace { regex, replace } => {
					let regex = Regex::new(regex).unwrap();

					extract = regex.replace_all(&extract, replace).to_string();
				} // crate::meta::Op::Swap { a, b } => {
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

pub fn extract_text(path: &impl AsRef<Path>, source_meta: &SourceMeta) -> Result<PdfExtract> {
	let pages = extract_pages(path)?;
	let sections = source_meta
		.sections
		.par_iter()
		.map(|section| make_section(&pages, section, false))
		.collect();

	Ok(PdfExtract { sections })
}

// Stage 2

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
	kind: PageKind,
	pub extract: String,
}

impl Section {
	pub fn parse(&self) -> Vec<Item> {
		self.kind.parse(&self.extract)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfExtract {
	sections: Vec<Section>,
	// errors: Vec<Error>,
}

impl PdfExtract {
	pub fn parse(&self) -> Vec<Item> {
		self.sections
			.par_iter()
			.flat_map(|span| span.parse())
			.collect()
	}
}
