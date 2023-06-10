use std::{collections::HashMap, fs::File, io::Write, ops::RangeFrom, path::Path};

use anyhow::Result;
use mupdf::Document;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
	meta::{MyRangeFrom, SourceMeta},
	page_kind::{Item, PageKind},
};

pub fn extract_text(path: &impl AsRef<Path>, source_meta: &SourceMeta) -> Result<PdfExtract> {
	let document = Document::open(path.as_ref().to_str().unwrap())?;

	let map: HashMap<usize, String> = document
		.pages()?
		.enumerate()
		.filter_map(|(i, p)| p.ok().map(|p| (i, p)))
		.filter_map(|(i, page)| page.to_text().ok().map(|p| (i, p)))
		.collect();

	let sections: Vec<_> = source_meta
		.sections
		.par_iter()
		.map(|section| {
			let vec: Vec<String> = (section.pages.clone())
				.into_par_iter()
				.filter_map(|i| map.get(&i))
				.flat_map(|page| {
					page.split("\n")
						.filter(|line| !line.is_empty())
						.collect::<Vec<_>>()
				})
				.map(str::to_owned)
				.collect();

			let extract = vec.join("\n");

			// {
			// 	let str = "\nWhere the Bodies Are Buried ";
			// 	let pos = extract.find(str);

			// 	if pos.is_some() {
			// 		println!("{section:?} {:?} {:?}", pos, pos.map(|p| p + str.len()));
			// 	}
			// }

			let mut extract = if let Some(range) = &section.range {
				match range {
					crate::meta::Span::Range(range) => extract[range.clone()].to_owned(),
					crate::meta::Span::From(range) => extract
						[<MyRangeFrom as Into<RangeFrom<usize>>>::into(range.clone())]
					.to_owned(),
				}
			} else {
				extract
			};

			for op in &section.ops {
				match op {
					crate::meta::Op::Insert { pos, char } => {
						extract.insert(*pos, *char);
					}
				}
			}

			Section {
				extract,
				kind: section.kind.clone(),
			}
		})
		.collect();

	Ok(PdfExtract { sections })
}

// Stage 2

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
	kind: PageKind,
	extract: String,
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
