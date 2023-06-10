use std::{
	fs::File,
	io::Write,
	path::{Path, PathBuf},
};

use anyhow::Result;
use pyo3::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
	def::SourceFileDef,
	page_kind::{Item, PageKind},
};

fn extract_text(path: &impl AsRef<Path>) -> Result<String> {
	let path = path.as_ref();
	let code = include_str!("py/extract_text.py");

	let str = Python::with_gil(|py| -> PyResult<String> {
		let extract = PyModule::from_code(py, code, "extract_text.py", "extract")?;
		let extract = extract.getattr("extract_text")?.call1((path,))?.extract()?;

		Ok(extract)
	})?;

	Ok(str)
}

pub fn extract(path: &impl AsRef<Path>, source_file_def: &SourceFileDef) -> Result<PdfExtract> {
	let text = extract_text(path)?;
	let path = path.as_ref();

	let str = "though you had it at whatever level she possesses.";
	let i = text.find(str);
	println!("{:?} {:?}", i, i.map(|i| i + str.len()));

	// let _ = File::create(&path.with_extension("txt"))?.write_all(text.as_bytes());

	let spans = source_file_def
		.spans
		.par_iter()
		.map(|span| PageSpan {
			extract: text[span.range.clone()].to_owned(),
			kind: span.kind.clone(),
		})
		.collect();

	Ok(PdfExtract { spans })
}

// Stage 2

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageSpan {
	kind: PageKind,
	extract: String,
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
