use std::{
	fs::File,
	hash::Hasher,
	io::{BufReader, Read},
	path::Path,
};

use anyhow::Result;
// use fasthash::XXHasher;
use highway::HighwayHasher;

pub fn hash_file(file: &File) -> Result<u64> {
	let mut reader = BufReader::new(file);

	let digest = {
		let mut hasher = HighwayHasher::default();
		let mut buffer = [0; 1024];
		loop {
			let count = reader.read(&mut buffer)?;
			if count == 0 {
				break;
			}
			hasher.write(&buffer[..count]);
		}
		hasher.finish()
	};
	Ok(digest)
}

pub fn hash(path: impl AsRef<Path>) -> Result<u64> {
	let file = File::open(path)?;

	hash_file(&file)
}
