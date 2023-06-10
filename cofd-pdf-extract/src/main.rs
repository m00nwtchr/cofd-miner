use std::{
	collections::HashMap,
	fs::{self, File},
	path::{Path, PathBuf},
	sync::RwLock,
};

use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

mod hash;
mod meta;
mod page_kind;
// mod parser;
mod source_file;

use meta::SourceMeta;
use source_file::{extract_text, PdfExtract};

// fn write_def() -> anyhow::Result<()> {
// 	let source_file_def = SourceFileDef {
// 		hash: 0x127A8AA22916FDCD,
// 		timestamp: 1462320000, // May 4, 2016
// 		spans: vec![PageSpanDef {
// 			range: 100..109,
// 			kind: PageKind::Merit,
// 		}],
// 	};

// 	serde_json::ser::to_writer_pretty(File::create("Def.json")?, &source_file_def)?;
// 	Ok(())
// }

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
		.map(|path| -> Result<SourceMeta> { Ok(serde_json::de::from_reader(File::open(path)?)?) })
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

	let sources: Vec<_> = paths
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
				println!("Unknown file: {}, Hash: {hash:X}", path.display());
			}

			o.map(|def| (path, def))
		})
		.collect();

	serde_json::ser::to_writer(File::create(cache_path)?, &cache.into_inner()?)?;

	sources
		.into_par_iter()
		.map(|(path, source_def)| -> Result<(PathBuf, PdfExtract)> {
			let json_path = out_path
				.join(path.file_name().unwrap())
				.with_extension("json");

			let pdf_extract = if json_path.exists() {
				serde_json::de::from_reader(File::open(&json_path)?)?
			} else {
				let pdf_extract = extract_text(&path, source_def)?;

				serde_json::ser::to_writer_pretty(File::create(&json_path)?, &pdf_extract)?;

				pdf_extract
			};

			Ok((json_path, pdf_extract))
		})
		.filter_map(|f| f.ok())
		.for_each(|(json_path, p)| {
			// println!("{p:?}");
			let vecs = p.parse();
			// println!("{vecs:?}");
			serde_json::ser::to_writer_pretty(
				File::create(json_path.with_extension("stage2.json")).unwrap(),
				&vecs,
			)
			.unwrap();
		});

	Ok(())
}
