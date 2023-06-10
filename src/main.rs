use std::{
	fs::{self, File},
	io::Write,
	path::{Path, PathBuf},
};

use anyhow::Result;
use rayon::prelude::*;

mod def;
mod hash;
mod page_kind;
// mod parser;
mod source_file;

mod schema;

use def::{PageSpanDef, SourceFileDef, TokenRange};
use page_kind::*;
use source_file::{extract, PdfExtract};
use walkdir::{DirEntry, WalkDir};

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

fn load_defs() -> Result<Vec<SourceFileDef>> {
	Ok(fs::read_dir("meta")?
		.into_iter()
		.filter_map(|entry| entry.ok().map(|e| e.path()))
		.filter(|path| {
			path.extension()
				.and_then(|ext| Some(ext.eq("json")))
				.unwrap_or(false)
		})
		.map(|path| -> Result<SourceFileDef> {
			Ok(serde_json::de::from_reader(File::open(path)?)?)
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
	let source_file_defs = vec![
		SourceFileDef {
			hash: 0x127A8AA22916FDCD,
			timestamp: 1462320000, // May 4, 2016
			spans: vec![
				PageSpanDef {
					range: 405544..432406,
					kind: PageKind::Merit(Some("Awakened".to_string())),
				},
				PageSpanDef {
					range: 432407..450302,
					kind: PageKind::Merit(None),
				},
				PageSpanDef {
					range: 1324269..1327325,
					kind: PageKind::Merit(Some("Sleeper".to_owned())),
				},
				PageSpanDef {
					range: 1340991..1347013,
					kind: PageKind::Merit(None),
				},
			],
		},
		SourceFileDef {
			hash: 0x9CC1F4CC8AA30AC2, // CofD
			timestamp: 1449878400,    // May 4, 2016
			spans: vec![PageSpanDef {
				range: 138923..240656,
				kind: PageKind::Merit(None),
			}],
		},
		SourceFileDef {
			hash: 0xD7036DF1B5C78357, // VtR 2e
			timestamp: 1387411200,
			spans: vec![
				PageSpanDef {
					range: 425794..470434,
					kind: PageKind::Merit(Some("Kindred".to_owned())),
				},
				PageSpanDef {
					range: 470765..502137,
					kind: PageKind::Merit(None),
				},
				PageSpanDef {
					range: 1279243..1289074,
					kind: PageKind::Merit(None),
				},
			],
		},
	];
	// let source_file_defs = load_defs()?;

	let out_path = Path::new("./out");

	let paths: Vec<PathBuf> = WalkDir::new("pdf")
		.into_iter()
		.filter_entry(|e| !is_hidden(e) && is_pdf(e))
		.filter_map(|e| e.ok())
		.map(|f| f.path().to_owned())
		.collect();

	let sources: Vec<_> = paths
		.into_par_iter()
		.filter_map(|path| hash::hash(&path).ok().map(|hash| (path, hash)))
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

	sources
		.into_par_iter()
		.map(|(path, source_def)| -> Result<(PathBuf, PdfExtract)> {
			let json_path = out_path
				.join(path.file_name().unwrap())
				.with_extension("json");

			let pdf_extract = if json_path.exists() {
				serde_json::de::from_reader(File::open(&json_path)?)?
			} else {
				let pdf_extract = extract(&path, source_def)?;

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
			serde_json::ser::to_writer(
				File::create(json_path.with_extension("stage2.json")).unwrap(),
				&vecs,
			)
			.unwrap();
		});

	Ok(())
}
