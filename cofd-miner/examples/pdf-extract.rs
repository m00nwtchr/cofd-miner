use std::{
	collections::HashMap,
	fs::File,
	path::{Path, PathBuf},
	sync::RwLock,
};

use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use walkdir::{DirEntry, WalkDir};

use cofd_miner::hash;

fn to_path_pretty<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
	let mut ser = serde_json::Serializer::with_formatter(
		File::create(path)?,
		PrettyFormatter::with_indent(b"\t"),
	);
	Ok(value.serialize(&mut ser)?)
}

#[derive(Default, Serialize, Deserialize)]
struct Cache {
	hash: HashMap<PathBuf, u64>,
	#[serde(default, skip)]
	dirty: bool,
}

fn is_hidden(entry: &DirEntry) -> bool {
	entry
		.file_name()
		.to_str()
		.map(|s| s.starts_with('.'))
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
	// env_logger::init();

	let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
	let pdf_path = std::fs::canonicalize(manifest_dir.join("../pdf/")).unwrap();

	let cache_path = manifest_dir.join("../cache.json");
	let cache = RwLock::new(if cache_path.exists() {
		serde_json::de::from_reader(File::open(&cache_path)?)?
	} else {
		Cache::default()
	});

	let out_path = &manifest_dir.join("res/tests/data/");
	if !out_path.exists() {
		std::fs::create_dir_all(out_path)?;
	}

	let paths: Vec<PathBuf> = WalkDir::new(pdf_path)
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
					let mut cache = cache.write().unwrap();
					cache.hash.insert(path.clone(), *hash);
					cache.dirty = true;
				}

				hash
			}
		})
		.flat_map(|(path, hash)| cofd_miner::get_meta(hash).map(|meta| (path, meta)))
		.flat_map(|(path, meta)| {
			cofd_miner::parse_book_with_meta(&path, meta).map(|book| (path, book))
		})
		.for_each(|(path, book)| {
			let json_path = out_path
				.join(path.file_name().unwrap())
				.with_extension("json");

			to_path_pretty(json_path, &book).unwrap();
		});

	if cache.read().unwrap().dirty {
		serde_json::ser::to_writer(File::create(cache_path)?, &cache.into_inner()?)?;
	}

	Ok(())
}
