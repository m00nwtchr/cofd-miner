use std::{fs::File, path::Path};

use codex_scraper::{self, download, parse, url_to_name, PageType};
use reqwest::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	let urls = [
		("https://codexofdarkness.com/wiki/Gifts", PageType::Gifts),
		// (
		// 	"https://codexofdarkness.com/wiki/Merits,_Universal",
		// 	PageType::Merits,
		// ),
		// (
		// 	"https://codexofdarkness.com/wiki/Merits,_Vampire",
		// 	PageType::Merits,
		// ),
	];

	let cache_path =
		Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var"))
			.join(".cache");

	let tasks: Vec<_> = urls
		.into_iter()
		.map(|(url, page)| {
			tokio::spawn({
				let cache_path = cache_path.clone();
				async move {
					let url = Url::parse(&url).expect("Invalid url");

					let text = download(&url, &cache_path).await.unwrap();
					let book = parse(&text, page).unwrap();

					let ron_path = cache_path.join(format!("{}.ron", url_to_name(&url)));
					ron::ser::to_writer(File::create(ron_path).unwrap(), &book).unwrap();
				}
			})
		})
		.collect();

	for task in tasks {
		let book = task.await?;
	}

	Ok(())
}

// fn main() {}
