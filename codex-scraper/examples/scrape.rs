// use std::path::Path;

// use codex_scraper::{self, download, parse, PageType};

// #[tokio::main(flavor = "current_thread")]
// async fn main() -> anyhow::Result<()> {
// 	let urls = [
// 		("https://codexofdarkness.com/wiki/Gifts", PageType::Gifts),
// 		(
// 			"https://codexofdarkness.com/wiki/Merits,_Universal",
// 			PageType::Merits,
// 		),
// 		// (
// 		// 	"https://codexofdarkness.com/wiki/Merits,_Vampire",
// 		// 	PageType::Merits,
// 		// ),
// 	];

// 	let cache_path =
// 		Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var"))
// 			.join(".cache");

// 	let tasks: Vec<_> = urls
// 		.into_iter()
// 		.map(|(url, page)| {
// 			tokio::spawn({
// 				let cache_path = cache_path.clone();
// 				async move {
// 					let text = download(url.to_owned(), cache_path).await.unwrap();
// 					parse(&text, page)
// 				}
// 			})
// 		})
// 		.collect();

// 	for task in tasks {
// 		let book = task.await??;

// 		println!("{book:?}");
// 	}

// 	Ok(())
// }

fn main() {}
