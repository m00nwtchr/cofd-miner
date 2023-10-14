use std::{collections::BTreeMap, ops::RangeFrom, path::Path, str::FromStr};

use anyhow::Result;
use either::Either;
use mupdf::Document;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use cofd_schema::{
	book::{Book, BookInfo},
	item::{Item, ItemProp, PropValue},
	prerequisites::Prerequisite,
};

use crate::{
	meta::{MyRangeFrom, SectionDefinition, SourceMeta},
	page_kind::PageKind,
};

pub fn extract_pages(path: &impl AsRef<Path>) -> Result<BTreeMap<usize, String>> {
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
	pages: &BTreeMap<usize, String>,
	section: &SectionDefinition,
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
				crate::meta::Op::Replace { range, replace } => {
					extract.replace_range(range.clone(), replace);
				}
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

	Ok(PdfExtract {
		info: source_meta.info,
		sections,
	})
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
	info: BookInfo,
	sections: Vec<Section>,
	// errors: Vec<Error>,
}

// #[serde_as]
// #[derive(Serialize)]
// pub struct PdfParse {
// 	info: BookInfo,
// 	merits: Vec<Item>,
// }

fn convert_properties(
	item_or_properties: &mut Either<&mut Item, &mut BTreeMap<ItemProp, PropValue>>,
) {
	for (prop, value) in match item_or_properties {
		Either::Left(item) => &mut item.properties,
		Either::Right(properties) => properties,
	} {
		match prop {
			ItemProp::Prerequisites => {
				if let PropValue::Vec(vec) = value {
					let mut prereqs = Vec::new();

					for str in vec {
						if let Ok(prereq) = Prerequisite::from_str(str) {
							prereqs.push(prereq);
						}
					}

					if !prereqs.is_empty() {
						*value = PropValue::Prerequisites(prereqs);
					}
				}
			}
			ItemProp::DicePool => {}
			_ => {}
		}
	}
	if let Either::Left(item) = item_or_properties {
		for child in &mut item.children {
			convert_properties(&mut Either::Right(&mut child.properties));
		}
	}
}

impl PdfExtract {
	pub fn parse(self) -> Book {
		let mut parse = Book {
			info: self.info,
			merits: Default::default(),
			mage_spells: Default::default(),
		};

		let sections: Vec<(PageKind, Vec<Item>)> = self
			.sections
			.par_iter()
			.map(|span| (span.kind.clone(), span.parse()))
			.map(|(kind, mut parsed)| {
				parsed
					.par_iter_mut()
					.for_each(|item| convert_properties(&mut Either::Left(item)));

				(kind, parsed)
			})
			.collect();

		for (kind, vec) in sections {
			match kind {
				PageKind::Merit(_) => parse.merits.extend(vec),
				PageKind::MageSpell => parse.mage_spells.extend(vec),
			}
		}
		parse.merits.sort_by(|a, b| a.name.cmp(&b.name));
		parse.mage_spells.sort_by(|a, b| a.name.cmp(&b.name));

		parse
	}
}
