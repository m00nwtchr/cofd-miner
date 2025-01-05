use std::str::FromStr;

use anyhow::Result;
use cofd_meta::PageKind;
use cofd_schema::{
	book::OtherGift,
	item::{
		gift::{Facet, Gift, Other},
		Item,
	},
	prelude::BookInfo,
	template::werewolf::Renown,
};
use convert_case::{Case, Casing};
use once_cell::sync::Lazy;
use regex::Regex;

use super::{get_book_reference, item::ItemProp};
use crate::{
	parse::{item::RawItem, to_paragraphs},
	source::Section,
};

static GIFT_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
	Regex::new(
		r"(?xmi)
			^
			(?P<name>[^\s.][^\n.]+)               # Name
			\s?
			\(
				(?P<renown>                       # Renown
					(Purity|Glory|Honor|Wisdom|Cunning)
				)
			\)
			\s?
			$
		",
	)
	.unwrap()
});

pub fn parse_gifts(info: &BookInfo, section: &Section) -> Result<Vec<OtherGift>> {
	let mut out = Vec::new();
	let mut facets = Vec::new();
	let mut body: Vec<&str> = Vec::new();

	let PageKind::Gift(kind) = section.kind else {
		unreachable!()
	};

	for line in section.extract.split('\n').rev() {
		let lower = line.to_ascii_lowercase();

		let last = body.last().map(|s| s.trim()).unwrap_or_default();
		let haystack = [line, last];

		if (lower.starts_with("gift") || lower.ends_with("gift")) && !lower.eq("shadow gifts") {
			out.push(Gift {
				name: line.to_case(Case::Title),
				facets,
				kind,
			});
			facets = Vec::new();
		} else if let Some(captures) = GIFT_HEADER_REGEX.captures(&haystack.join("\n")) {
			if last.starts_with('(') && last.ends_with(')') {
				body.pop();
			}

			let name = captures
				.name("name")
				.unwrap()
				.as_str()
				.trim()
				.to_case(Case::Title);
			let renown = Renown::from_str(captures.name("renown").unwrap().as_str().trim())?;
			let reference = get_book_reference(&captures, section, info);

			body.reverse();
			let mut raw_item = RawItem::try_from(body)?;

			facets.push(Item {
				name: name.clone(),
				reference,
				description: to_paragraphs(raw_item.get(None)),
				effects: raw_item.take(Some(ItemProp::Effects)),
				inner: Facet {
					action: raw_item.action(),
					inner: Other { renown },
				},
			});

			body = Vec::new();
		} else {
			body.push(line);
		}
	}

	Ok(out)
}
