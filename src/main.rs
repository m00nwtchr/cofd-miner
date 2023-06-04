use std::{
	fs::{self, File},
	path::PathBuf,
};

use anyhow::Result;
use rayon::prelude::*;

mod def;
mod hash;
mod page_kind;
mod source_file;

mod schema;

use def::{PageSpanDef, SourceFileDef};
use page_kind::*;
use source_file::{PdfExtract, SourceFile};

type PageIndex = u32;

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

fn main() -> anyhow::Result<()> {
	let source_file_defs = vec![
		SourceFileDef {
			hash: 0x127A8AA22916FDCD,
			timestamp: 1462320000, // May 4, 2016
			spans: vec![PageSpanDef {
				range: 100..109,
				kind: PageKind::Merit,
			}],
		},
		SourceFileDef {
			hash: 0x9CC1F4CC8AA30AC2,
			timestamp: 1449878400, // May 4, 2016
			spans: vec![PageSpanDef {
				range: 45..67,
				kind: PageKind::Merit,
			}],
		},
	];
	// let source_file_defs = load_defs()?;

	let sources: Vec<_> = fs::read_dir("pdf")?
		.filter_map(|f| f.ok())
		.map(|f| f.path())
		.filter(|path| {
			path.extension()
				.and_then(|ext| Some(ext.eq("pdf")))
				.unwrap_or(false)
		})
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
			let json_path = path.with_extension("json");

			let pdf_extract = if json_path.exists() {
				serde_json::de::from_reader(File::open(&json_path)?)?
			} else {
				let mut source = SourceFile::from(path);
				source.load_pdf()?;

				let pdf_extract = source.extract(source_def)?;
				serde_json::ser::to_writer_pretty(File::create(&json_path)?, &pdf_extract)?;
				pdf_extract
			};

			Ok((json_path, pdf_extract))
		})
		.filter_map(|f| f.ok())
		.for_each(|(json_path, p)| {
			//
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
