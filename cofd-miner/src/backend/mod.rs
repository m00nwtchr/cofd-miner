// #[cfg(feature = "lopdf")]
// mod lopdf;
// #[cfg(feature = "lopdf")]
// pub use lopdf::extract_pages;

#[cfg(feature = "mupdf")]
mod mupdf;

use std::collections::BTreeMap;

#[cfg(feature = "mupdf")]
pub use mupdf::extract_pages;

pub type PdfText = BTreeMap<usize, Vec<String>>;
