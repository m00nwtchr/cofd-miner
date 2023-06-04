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

use def::{PageSpanDef, SourceFileDef};
use page_kind::*;
use source_file::{PdfExtract, SourceFile};

type PageIndex = u32;

fn main() -> anyhow::Result<()> {
	let source_file_defs = vec![SourceFileDef {
		hash: 0x127A8AA22916FDCD,
		spans: vec![PageSpanDef {
			range: 100..109,
			kind: PageKind::Merit,
		}],
	}];

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
				println!("Unknown file: {}, Hash: {hash}", path.display());
			}
			o.map(|def| (path, def))
		})
		.collect();

	sources
		.into_par_iter()
		.map(
			|(path, source_def)| -> Result<(PathBuf, PdfExtract, bool)> {
				let json_path = path.with_extension("json");

				let flag;
				let pdf_extract = if json_path.exists() {
					flag = false;
					serde_json::de::from_reader(File::open(&json_path)?)?
				} else {
					flag = true;
					let mut source = SourceFile::from(path);
					source.load_pdf()?;
					source.extract(source_def)?
				};

				Ok((json_path, pdf_extract, flag))
			},
		)
		.filter_map(|f| f.ok())
		.map(
			|(json_path, pdf_extract, should_save)| -> Result<PdfExtract> {
				if should_save {
					serde_json::ser::to_writer_pretty(File::create(json_path)?, &pdf_extract)?;
				}

				Ok(pdf_extract)
			},
		)
		.filter_map(|f| f.ok())
		.for_each(|p| {
			//
			// println!("{p:?}");
			let vecs = p.parse();
			println!("{vecs:?}");
		});

	Ok(())
}
