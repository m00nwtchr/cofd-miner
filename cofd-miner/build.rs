use std::{
	env,
	fs::{self, File},
	path::Path,
};

use anyhow::anyhow;

use cofd_meta::SourceMeta;

fn main() {
	println!("cargo:rerun-if-changed=../meta");
	let out_dir = env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("meta.bin");
	let vec: Vec<SourceMeta> = fs::read_dir("../meta")
		.unwrap()
		.filter_map(|entry| entry.ok().map(|e| e.path()))
		.filter(|path| path.extension().map(|ext| ext.eq("json")).unwrap_or(false))
		.map(|path| {
			serde_json::from_reader(File::open(&path).unwrap())
				.map_err(|err| anyhow!("{}: {}", path.display(), err))
				.unwrap()
		})
		.collect();

	rmp_serde::encode::write_named(&mut File::create(dest_path).unwrap(), &vec).unwrap();
}
