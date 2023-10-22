// #[cfg(feature = "lopdf")]
// mod lopdf;
// #[cfg(feature = "lopdf")]
// pub use lopdf::extract_pages;

#[cfg(feature = "mupdf")]
mod mupdf;
#[cfg(feature = "mupdf")]
pub use mupdf::extract_pages;
