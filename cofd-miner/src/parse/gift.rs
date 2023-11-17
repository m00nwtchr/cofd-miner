use std::str::FromStr;

use anyhow::Result;
use cofd_schema::{
	book::OtherGift,
	dice_pool,
	item::{
		gift::{Facet, Gift, Other},
		Item,
	},
	prelude::BookInfo,
	splat::werewolf::Renown,
};
use convert_case::Casing;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{parse::to_paragraphs, source::Section};

use super::{get_book_reference, item::ItemProp, PROP_REGEX};

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
	let mut body = Vec::new();

	for line in section.extract.split('\n').rev() {
		if line.to_ascii_lowercase().contains("gift") {
			out.push(Gift {
				name: line.to_string(),
				facets,
			});
			facets = Vec::new();
		} else if let Some(captures) = GIFT_HEADER_REGEX.captures(line) {
			let name = captures
				.name("name")
				.unwrap()
				.as_str()
				.trim()
				.to_case(convert_case::Case::Title);
			let renown = Renown::from_str(captures.name("renown").unwrap().as_str().trim())?;
			let reference = get_book_reference(&captures, section, info);

			// let cost = Vec::new();
			// let dice_pool = None;
			// let action = Vec::new();
			// let duration = Vec::new();

			let mut effects = Vec::new();
			let mut description = Vec::new();

			let mut flag = true;
			for line in body {
				if let Some(prop) = PROP_REGEX.captures(line) {
					flag = false;
					if let (Some(prop_key), Some(prop_val)) = (prop.get(1), prop.get(2)) {
						let prop_key = ItemProp::from_str(prop_key.as_str()).unwrap();

						match prop_key {
							ItemProp::Cost => todo!(),
							ItemProp::DicePool => todo!(),
							ItemProp::Action => todo!(),
							ItemProp::Duration => todo!(),
							_ => {}
						}
					}
				} else if flag {
					effects.push(line.to_string());
				} else {
					description.push(line.to_string());
				}
			}
			description.reverse();
			effects.reverse();

			facets.push(Item {
				name: name.to_string(),
				reference,
				description: to_paragraphs(description),
				effects: to_paragraphs(effects),
				inner: Facet {
					action: None,
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
