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
	splat::werewolf::Renown,
};
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use regex::Regex;

use super::{get_book_reference, item::ItemProp, process_action, PROP_REGEX};
use crate::{parse::to_paragraphs, source::Section};

lazy_static! {
	static ref GIFT_HEADER_REGEX: Regex = Regex::new(
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
		"
	)
	.unwrap();
}

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

			let mut lines = Vec::new();

			let mut action = None;
			let mut description = Vec::new();
			let mut effects = Vec::new();

			let mut first_prop = true;

			for line in body {
				if let Some(prop) = PROP_REGEX.captures(line.trim_start()) {
					if let (Some(prop_key), Some(prop_val)) = (prop.get(1), prop.get(2)) {
						let prop_key = ItemProp::from_str(prop_key.as_str()).unwrap();
						let line = prop_val.as_str();

						if !effects.is_empty() {
							first_prop = false;
						}

						lines.push(line.to_owned());
						lines.reverse();
						process_action(&mut action, prop_key, to_paragraphs(lines));

						lines = Vec::new();
					}
				} else if line.starts_with('\t') {
					lines.push(line.to_owned());
					if first_prop {
						effects.extend(lines);
					} else {
						description.extend(lines);
					}
					lines = Vec::new();
				} else {
					lines.push(line.to_owned());
				}
			}

			if !lines.is_empty() {
				description.extend(lines);
			}
			description.reverse();
			effects.reverse();

			facets.push(Item {
				name: name.clone(),
				reference,
				description: to_paragraphs(description),
				effects: to_paragraphs(effects),
				inner: Facet {
					action,
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
