use std::{
	collections::{BTreeMap, HashMap},
	ops::RangeBounds,
	path::PathBuf,
};

use anyhow::{anyhow, Result};
use lopdf::{content::Content, encodings::Encoding, Document, Object, ObjectId};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
	def::{PageSpanDef, SourceFileDef},
	page_kind::{Item, PageKind},
	PageIndex,
};

static IGNORE: &[&str] = &[
	"Length",
	"BBox",
	"FormType",
	"Matrix",
	// "Resources",
	"Type",
	"XObject",
	"Subtype",
	"Filter",
	"ColorSpace",
	"Width",
	"Height",
	"BitsPerComponent",
	"Length1",
	"Length2",
	"Length3",
	"PTEX.FileName",
	"PTEX.PageNumber",
	"PTEX.InfoDict",
	"FontDescriptor",
	"ExtGState",
	// "Font",
	"MediaBox",
	"Annot",
];

pub struct SourceFile {
	path: PathBuf,
	document: Option<Document>,
	pages: Option<BTreeMap<PageIndex, ObjectId>>,
}

impl From<PathBuf> for SourceFile {
	fn from(path: PathBuf) -> Self {
		SourceFile {
			path,
			document: None,
			pages: None,
		}
	}
}

fn extract_text<R: RangeBounds<u32> + Sync + Send>(
	source_file: &SourceFile,
	pages: &R,
) -> anyhow::Result<BTreeMap<u32, Vec<String>>> {
	fn collect_text(
		text: &mut String,
		encoding: Option<&Encoding>,
		operands: &[Object],
	) -> Result<()> {
		for operand in operands.iter() {
			match *operand {
				Object::String(ref bytes, _) => {
					text.push_str(&Document::decode_text(encoding, bytes)?);
				}
				Object::Array(ref arr) => {
					collect_text(text, encoding, arr)?;
					text.push(' ');
				}
				Object::Integer(i) => {
					if i < -100 {
						text.push(' ');
					}
				}
				_ => {}
			}
		}
		Ok(())
	}
	let Some(document) = &source_file.document else {
		return Err(anyhow!(""))
	};
	let Some(page_map) = &source_file.pages else {
		return Err(anyhow!(""))
	};

	let pdftext: BTreeMap<_, _> = page_map
		.into_par_iter()
		.filter(|(num, _)| pages.contains(num))
		.map(|(page_num, page_id)| {
			let fonts = document.get_page_fonts(*page_id);
			let encodings: HashMap<_, _> = fonts
				.into_par_iter()
				.filter_map(|(name, font)| {
					font.get_font_encoding(document).ok().map(|enc| (name, enc))
				})
				.collect();

			let content_data = document.get_page_content(*page_id).unwrap();
			let content = Content::decode(&content_data).unwrap();
			let mut current_encoding = None;

			let mut text = String::new();
			let mut vec = Vec::new();
			let mut flag = false;

			for operation in &content.operations {
				match operation.operator.as_ref() {
					"Tf" => {
						let current_font = operation
							.operands
							.get(0)
							.ok_or_else(|| anyhow::anyhow!("Missing font operand"))
							.unwrap()
							.as_name()
							.unwrap();
						current_encoding = encodings.get(current_font);
						// println!("{:?}", current_encoding);
					}
					"Tj" | "TJ" => {
						collect_text(&mut text, current_encoding, &operation.operands);
					}
					"ET" => {
						if !text.ends_with('\n') {
							text.push('\n');
							let txt = text.trim();

							if txt.eq("-") {
								flag = true;
							} else {
								if flag || txt.starts_with('-') {
									*vec.last_mut().unwrap() += txt;
									flag = false;
								} else {
									if !text.is_empty() {
										vec.push(txt.to_owned());
									}
								}
							}
							text.clear();
						} else {
							print!("Newline: {text}");
						}
					}
					_ => {}
				}
			}

			(*page_num, vec)
		})
		.collect();

	Ok(pdftext)
}

impl SourceFile {
	pub fn load_pdf(&mut self) -> anyhow::Result<()> {
		let document = Document::load_filtered(&self.path, SourceFile::_filter_func)?;
		let pages = document.get_pages();

		self.document = Some(document);
		self.pages = Some(pages);
		Ok(())
	}

	pub fn extract_span(&self, def: &PageSpanDef) -> anyhow::Result<PageSpan> {
		let extract = extract_text(&self, &def.range)?;
		let extract = extract.into_iter().flat_map(|(_, v)| v).collect();

		let pages = PageSpan {
			extract,
			kind: def.kind.clone(),
		};

		Ok(pages)
	}

	pub fn extract(self, source_file_def: &SourceFileDef) -> anyhow::Result<PdfExtract> {
		let spans: Vec<_> = source_file_def
			.spans
			.par_iter()
			.filter_map(|span| self.extract_span(span).ok())
			.collect();

		Ok(PdfExtract { spans })
	}

	fn _filter_func(object_id: (u32, u16), object: &mut Object) -> Option<((u32, u16), Object)> {
		if object
			.type_name()
			.map_or(false, |name| IGNORE.contains(&name))
		{
			return None;
		}
		if let Ok(d) = object.as_dict_mut() {
			// d.remove(b"Font");
			// d.remove(b"Resources");
			d.remove(b"Producer");
			d.remove(b"ModDate");
			d.remove(b"Creator");
			d.remove(b"ProcSet");
			d.remove(b"XObject");
			d.remove(b"MediaBox");
			d.remove(b"Annots");
			if d.is_empty() {
				return None;
			}
		}
		Some((object_id, object.to_owned()))
	}
}

// Stage 2

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageSpan {
	kind: PageKind,
	extract: Vec<String>,
}

impl PageSpan {
	pub fn parse(&self) -> Vec<Item> {
		self.kind.parse(&self.extract)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfExtract {
	spans: Vec<PageSpan>,
	// errors: Vec<Error>,
}

impl PdfExtract {
	pub fn parse(&self) -> Vec<Item> {
		self.spans
			.par_iter()
			.flat_map(|span| span.parse())
			.collect()
	}
}
