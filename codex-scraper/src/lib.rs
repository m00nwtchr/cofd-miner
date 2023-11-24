use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cofd_schema::book::{Book, BookId};
use cofd_schema::prelude::BookInfo;
use itertools::Itertools;
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};

mod gifts;
mod merits;

pub enum PageType {
	Gifts,
	Merits,
}

type MultiMap = HashMap<String, Vec<Vec<String>>>;

pub async fn download<P: AsRef<Path>>(url: String, cache_path: P) -> anyhow::Result<String> {
	let url = Url::parse(&url).expect("Invalid url");

	let page_name = url
		.path_segments()
		.expect("Path segments")
		.last()
		.expect("Last path segment")
		.to_string();

	let cache_path = cache_path.as_ref();
	if !cache_path.exists() {
		std::fs::create_dir_all(cache_path)?;
	}

	let name = page_name.replace(',', "");
	let html_path = cache_path.join(format!("{name}.html"));

	let text;
	if !html_path.exists() {
		log::info!("Downloading: {url}");

		let resp = reqwest::get(url).await?;
		text = resp.text().await?;

		fs::write(html_path, &text)?;
		Ok(text)
	} else {
		Ok(fs::read_to_string(html_path)?)
	}
}

pub fn parse(text: &str, page: PageType) -> anyhow::Result<Book> {
	let document = Html::parse_document(text);

	let selector = Selector::parse(".mw-parser-output > section, h2").unwrap();
	let table_sel = Selector::parse("table").unwrap();

	let mut map = HashMap::new();
	for (header, section) in document.select(&selector).skip(1).tuples() {
		let title = header.text().last().expect("Last text in header");
		let table = section.select(&table_sel).next().unwrap();

		for tr in table
			.children()
			.last()
			.unwrap()
			.children()
			.filter_map(ElementRef::wrap)
			.skip(1)
		{
			let mut vec = Vec::new();
			for td in tr.children().filter_map(ElementRef::wrap) {
				vec.push(
					td.inner_html()
						.trim()
						.to_string()
						.replace("<i>", "")
						.replace("</i>", ""),
				);
			}

			map.entry(title.to_string()).or_insert(Vec::new()).push(vec);
		}
	}

	let mut book = Book::from(BookInfo::new("Codex of Darkness".to_owned(), BookId::Codex));

	match page {
		PageType::Gifts => {
			let (moon_gifts, gifts) = gifts::parse_gifts(map)?;

			book.moon_gifts.extend(moon_gifts);
			book.gifts.extend(gifts);
		}
		PageType::Merits => book.merits.extend(merits::parse_merits(map)?),
	};

	Ok(book)
}
