use std::{
	env,
	fs::{self, File},
	path::Path,
};

use anyhow::{anyhow, Result};

#[cfg(feature = "embed_meta")]
use cofd_meta::SourceMeta;

#[cfg(feature = "embed_meta")]
fn embed_meta() -> Result<()> {
	println!("cargo:rerun-if-changed=../../meta");
	let out_dir = env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("meta.bin");
	let vec: Vec<SourceMeta> = fs::read_dir("../../meta")
		.unwrap()
		.filter_map(|entry| entry.ok().map(|e| e.path()))
		.filter(|path| path.extension().map(|ext| ext.eq("json")).unwrap_or(false))
		.map(|path| {
			serde_json::from_reader(File::open(&path).unwrap())
				.map_err(|err| anyhow!("{}: {}", path.display(), err))
		})
		.collect::<Result<_>>()?;

	rmp_serde::encode::write_named(&mut File::create(dest_path).unwrap(), &vec)?;
	Ok(())
}

fn main() -> Result<()> {
	#[cfg(feature = "embed_meta")]
	embed_meta()?;

	Ok(())
}
