use std::{
	collections::HashMap,
	fs::{self, File},
	path::{Path, PathBuf},
	sync::RwLock,
};

use anyhow::Result;
use log::debug;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use walkdir::{DirEntry, WalkDir};

mod hash;
mod meta;
mod page_kind;
// mod parser;
mod source_file;

use meta::SourceMeta;
use source_file::{extract_text, PdfExtract};

fn to_path_pretty<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
	let mut ser = serde_json::Serializer::with_formatter(
		File::create(path).unwrap(),
		PrettyFormatter::with_indent(b"\t"),
	);
	Ok(value.serialize(&mut ser)?)
}

#[derive(Default, Serialize, Deserialize)]
struct Cache {
	hash: HashMap<PathBuf, u64>,
}

fn load_defs() -> Result<Vec<SourceMeta>> {
	Ok(fs::read_dir("meta")?
		.into_iter()
		.filter_map(|entry| entry.ok().map(|e| e.path()))
		.filter(|path| {
			path.extension()
				.and_then(|ext| Some(ext.eq("json")))
				.unwrap_or(false)
		})
		.map(|path| -> Result<SourceMeta> {
			let meta = serde_json::de::from_reader(File::open(&path)?)?;
			// serde_json::ser::to_writer_pretty(File::create(&path)?, &j)?;
			Ok(meta)
		})
		.filter_map(|r| r.ok())
		.collect())
}

fn is_hidden(entry: &DirEntry) -> bool {
	entry
		.file_name()
		.to_str()
		.map(|s| s.starts_with("."))
		.unwrap_or(false)
}

fn is_pdf(entry: &DirEntry) -> bool {
	if entry.file_type().is_file() {
		entry
			.file_name()
			.to_str()
			.map(|s| s.ends_with(".pdf"))
			.unwrap_or(false)
	} else {
		true
	}
}

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let cache_path = Path::new("cache.json");
	let cache = RwLock::new(if cache_path.exists() {
		serde_json::de::from_reader(File::open(cache_path)?)?
	} else {
		Cache::default()
	});

	let source_file_defs = load_defs()?;

	let out_path = Path::new("./out");
	let paths: Vec<PathBuf> = WalkDir::new("pdf")
		.into_iter()
		.filter_entry(|e| !is_hidden(e) && is_pdf(e))
		.filter_map(|e| e.ok())
		.map(|f| f.path().to_owned())
		.collect();

	paths
		.into_par_iter()
		.filter_map(|path| {
			if let Ok(Some(hash)) = cache.read().map(|c| c.hash.get(&path).cloned()) {
				Some((path, hash))
			} else {
				let hash = hash::hash(&path).ok().map(|hash| (path, hash));

				if let Some((path, hash)) = &hash {
					cache.write().unwrap().hash.insert(path.clone(), *hash);
				}

				hash
			}
		})
		.filter_map(|(path, hash)| {
			let o = source_file_defs
				.par_iter()
				.find_any(|def| def.hash.eq(&hash));
			if o.is_none() {
				debug!("Unknown file: {}, Hash: {hash:X}", path.display());
			}

			o.map(|def| (path, def))
		})
		.map(|(path, source_def)| -> Result<(PathBuf, PdfExtract)> {
			let json_path = out_path
				.join(path.file_name().unwrap())
				.with_extension("json");

			let pdf_extract = if json_path.exists() {
				serde_json::de::from_reader(File::open(&json_path)?)?
			} else {
				let pdf_extract = extract_text(&path, source_def)?;

				to_path_pretty(&json_path, &pdf_extract)?;

				pdf_extract
			};

			Ok((json_path, pdf_extract))
		})
		.filter_map(|f| f.ok())
		.for_each(|(json_path, p)| {
			// println!("{p:?}");
			let vecs = p.parse();
			// println!("{vecs:?}");

			to_path_pretty(json_path.with_extension("stage2.json"), &vecs).unwrap();
		});

	serde_json::ser::to_writer(File::create(cache_path)?, &cache.into_inner()?)?;

	Ok(())
}
