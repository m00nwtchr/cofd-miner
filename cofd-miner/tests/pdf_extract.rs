// #[derive(Default, Serialize, Deserialize)]
// struct Cache {
// 	hash: HashMap<PathBuf, u64>,
// 	#[serde(default, skip)]
// 	dirty: bool,
// }

// fn to_path_pretty<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
// 	let mut ser = serde_json::Serializer::with_formatter(
// 		File::create(path)?,
// 		PrettyFormatter::with_indent(b"\t"),
// 	);
// 	Ok(value.serialize(&mut ser)?)
// }
//
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
	entry
		.file_name()
		.to_str()
		.map(|s| s.starts_with('.'))
		.unwrap_or(false)
}

fn is_pdf(entry: &walkdir::DirEntry) -> bool {
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

fn is_data(entry: &DirEntry) -> bool {
	entry.file_type().is_ok_and(|f| f.is_file())
		&& entry
			.file_name()
			.to_str()
			.map(|s| !s.starts_with(".") && s.ends_with(".json"))
			.unwrap_or(false)
}

use std::fs::{DirEntry, File};
use std::path::Path;

use anyhow::{anyhow, Result};
use cofd_miner::hash;
use cofd_schema::book::Book;
use itertools::Itertools;
use rayon::prelude::*;
use walkdir::WalkDir;

#[test]
#[ignore]
fn pdf_extract() -> Result<()> {
	let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
	let data_path = manifest_dir.join("res/tests/data");
	let pdf_path = manifest_dir.join("../pdf/");
	if !data_path.exists() || !pdf_path.exists() {
		return Err(anyhow!("Test data doesn't exist, skipped."));
	}

	let data: Vec<Book> = data_path
		.read_dir()?
		.filter_map(Result::ok)
		.filter(is_data)
		.filter_map(|f| File::open(f.path()).ok())
		.filter_map(|f| serde_json::from_reader(f).ok())
		.collect();

	WalkDir::new(pdf_path)
		.into_iter()
		.filter_entry(|e| !is_hidden(e) && is_pdf(e))
		.par_bridge()
		.filter_map(Result::ok)
		.filter_map(|entry| hash::hash(entry.path()).ok().map(|hash| (entry, hash)))
		.filter_map(|(entry, hash)| {
			data.iter()
				.find_position(|b| b.info.hash.eq(&hash))
				.map(|b| (entry, hash, b.0))
		})
		.flat_map(|(entry, hash, i)| cofd_miner::get_meta(hash).map(|meta| (entry, meta, i)))
		.flat_map(|(entry, meta, i)| {
			cofd_miner::parse_book_with_meta(entry.path(), meta).map(|book| (book, i))
		})
		.for_each(|(result, data_index)| {
			let data = &data[data_index];

			let l = serde_json::to_string_pretty(&result).unwrap();
			let r = serde_json::to_string_pretty(data).unwrap();

			similar_asserts::assert_eq!(l, r);
		});

	Ok(())
}
