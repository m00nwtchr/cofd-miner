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

use cofd_meta_schema::SourceMeta;
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
	extract_text(path, source).map(parse::PdfExtract::parse)
}

pub fn parse_book(path: impl AsRef<Path>) -> anyhow::Result<Book> {
	let hash = hash(&path)?;
	let meta = get_meta(hash).ok_or(CofDMinerError::NoSuchMeta)?;

	parse_book_with_meta(path, meta)
}
