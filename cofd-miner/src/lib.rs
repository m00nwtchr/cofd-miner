#![deny(clippy::pedantic)]
#![allow(
	clippy::missing_errors_doc,
	clippy::module_name_repetitions,
	clippy::similar_names
)]
use std::path::Path;

use error::CofDMinerError;
use lazy_static::lazy_static;
pub use source::{extract_pages, extract_text, process_section};

use cofd_meta::SourceMeta;
use cofd_schema::book::Book;
use hash::hash;

mod backend;
mod parser_item;

pub mod error;
pub mod hash;
pub mod parse;
pub mod source;

const META_JSON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/meta.bin"));

lazy_static! {
	static ref META: Vec<SourceMeta> = rmp_serde::decode::from_slice(META_JSON).unwrap();
}

#[must_use]
pub fn get_meta(hash: u64) -> Option<&'static SourceMeta> {
	META.iter().find(|source| source.info.hash.eq(&hash))
}

pub fn parse_book_with_meta(path: impl AsRef<Path>, source: &SourceMeta) -> anyhow::Result<Book> {
	extract_text(path, source).and_then(parse::PdfExtract::parse)
}

pub fn parse_book(path: impl AsRef<Path>) -> anyhow::Result<Book> {
	let hash = hash(&path)?;
	let meta = get_meta(hash).ok_or(CofDMinerError::NoSuchMeta)?;

	parse_book_with_meta(path, meta)
}

#[cfg(test)]
mod test {
	use crate::parse_book;
	use cofd_schema::book::Book;

	#[test]
	fn roundtrip() {
		let book = parse_book("../pdf/Mage/Mage the Awakening 2e.pdf").unwrap();

		let book: Book =
			serde_json::de::from_str(&serde_json::ser::to_string(&book).unwrap()).unwrap();
		println!("RON");
		// let book: Book = ron::de::from_str(&ron::ser::to_string(&book).unwrap()).unwrap();
	}
}
