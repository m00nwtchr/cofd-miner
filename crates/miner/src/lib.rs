#![warn(clippy::pedantic)]
#![allow(
	clippy::missing_errors_doc,
	clippy::module_name_repetitions,
	clippy::similar_names
)]

use std::path::Path;

use cofd_meta::SourceMeta;
use cofd_schema::{book::Book, DOT_CHAR};
use error::CofDMinerError;
use hash::hash;
use once_cell::sync::Lazy;
use parse::PdfExtract;
use regex::Regex;

mod backend;

pub mod error;
pub mod hash;
pub mod parse;
pub mod source;

pub use source::{extract_pages, extract_text, process_section};

#[cfg(feature = "embed_meta")]
const META_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/meta.bin"));

#[cfg(feature = "embed_meta")]
static META: Lazy<Vec<SourceMeta>> =
	Lazy::new(|| rmp_serde::decode::from_slice(META_BYTES).unwrap());

static DOT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!("^{DOT_CHAR}+ ")).unwrap());

pub fn parse_book_with_meta(path: impl AsRef<Path>, source: &SourceMeta) -> anyhow::Result<Book> {
	extract_text(path, source).and_then(PdfExtract::parse)
}

#[must_use]
#[cfg(feature = "embed_meta")]
pub fn get_meta(hash: u64) -> Option<&'static SourceMeta> {
	META.iter().find(|source| source.info.hash.eq(&hash))
}

#[cfg(feature = "embed_meta")]
pub fn parse_book(path: impl AsRef<Path>) -> anyhow::Result<Book> {
	let hash = hash(&path)?;
	let meta = get_meta(hash).ok_or(CofDMinerError::NoSuchMeta)?;

	parse_book_with_meta(path, meta)
}
