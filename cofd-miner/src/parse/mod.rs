use std::{collections::HashMap, ops::Range};

use anyhow::Result;
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use cofd_meta::PageKind;
use cofd_schema::{
	book::{Book, BookInfo, BookReference},
	item::ActionFields,
};

mod item;
mod merit;

use crate::source::Section;

use self::{
	item::{convert_dice_pool, ItemProp},
	merit::parse_merits,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfExtract {
	pub info: BookInfo,
	pub sections: Vec<Section>,
	// errors: Vec<Error>,
}

impl PdfExtract {
	pub fn parse(self) -> Result<Book> {
		let mut parse = Book::from(self.info);

		for section in self.sections {
			match section.kind {
				PageKind::Merit(_) => parse.merits.extend(parse_merits(&parse.info, &section)?),
				_ => todo!()
				// PageKind::MageSpell => parse.mage_spells.extend(vec.into_iter().map(|i| match i {
				// 	_ => unreachable!(),
				// })),
				// PageKind::Gift(kind) => match kind {
				// 	GiftKind::Moon => parse.moon_gifts.extend(vec.into_iter().map(|i| match i {
				// 		ItemKind::MoonGift(item) => item,
				// 		_ => unreachable!(),
				// 	})),
				// 	GiftKind::Other => parse.gifts.extend(vec.into_iter().map(|i| match i {
				// 		ItemKind::OtherGift(item) => item,
				// 		_ => unreachable!(),
				// 	})),
				// },
			}
		}
		parse.merits.sort_by(|a, b| a.name.cmp(&b.name));
		parse.mage_spells.sort_by(|a, b| a.name.cmp(&b.name));

		Ok(parse)
	}
}

lazy_static! {

	static ref PROP_REGEX: Regex = Regex::new(r"^(Prerequisite|Style Tag|Cost|Dice Pool|Action|Duration|Effect|Drawback|Note)s?:\s?(.*)$").unwrap();
	//
}

// pub fn parse_span(info: &BookInfo, span: &Section) -> Result<Vec<ItemKind>> {
// 	// let mut out = Vec::new();
// 	// let mut str_pos = span.extract.len();

// 	Ok(out)
// }

fn get_book_reference(captures: &Captures<'_>, span: &Section, info: &BookInfo) -> BookReference {
	let page = if let Some(match_) = captures.get(0) {
		get_page_number(&span.page_ranges, match_.start())
	} else {
		unreachable!()
	};
	BookReference(info.id, *page)
}

fn process_action(action: &mut ActionFields, prop_key: ItemProp, lines: Vec<String>) -> bool {
	let mut flag = true;
	match prop_key {
		ItemProp::Action => action.action = lines,
		ItemProp::Cost => action.cost = lines,
		ItemProp::DicePool => action.dice_pool = convert_dice_pool(&lines),
		ItemProp::Duration => action.duration = lines,
		_ => flag = false,
	}

	flag
}

// TODO: Some edge cases don't get merged properly but works ok overall.
fn to_paragraphs(vec: Vec<String>) -> Vec<String> {
	let mut out = Vec::new();
	let mut paragraph = String::new();

	let mut flag = false;
	for line in vec {
		if !paragraph.is_empty() && !flag {
			paragraph.push(' ');
		} else if flag {
			flag = false;
		}

		paragraph.push_str(if line.ends_with("God-") {
			flag = true;
			&line
		} else if line.ends_with('-') {
			flag = true;
			line.trim_end_matches('-')
		} else {
			&line
		});

		if line.ends_with('.') {
			out.push(paragraph);
			paragraph = String::new();
		}
	}
	if !paragraph.is_empty() {
		out.push(paragraph);
	}

	out
}

fn get_body(str_pos: &mut usize, span: &str, captures: &Captures<'_>) -> Vec<String> {
	let body = span[captures.get(0).unwrap().end()..*str_pos]
		.split('\n')
		.filter_map(filter_normalize)
		.collect();
	*str_pos = captures.get(0).unwrap().start();

	body
}

fn get_page_number(page_ranges: &HashMap<usize, Range<usize>>, pos: usize) -> &usize {
	page_ranges
		.iter()
		.find(|(_, range)| range.contains(&pos))
		.map_or(&0, |(p, _)| p)
}

fn parse_name(captures: &Captures<'_>) -> String {
	let name = normalize(captures.name("name").map_or("", |f| f.as_str()));

	if name.chars().all(|f| f.is_uppercase() || !f.is_alphabetic()) {
		let is: Vec<_> = name
			.chars()
			.enumerate()
			.filter(|(_, c)| c.eq(&'-'))
			.map(|(i, _)| i)
			.collect();

		if is.is_empty() {
			name.to_case(Case::Title)
		} else {
			let mut name = name.to_case(Case::Title);
			for i in is {
				name.replace_range(i..=i, "-");
			}
			name
		}
	} else {
		name.to_string()
	}
}

fn normalize(str: &str) -> String {
	str.trim()
		.replace('\n', " ")
		.replace("  ", " ")
		.replace(['‘', '’'], "'")
}

fn filter_normalize(str: &str) -> Option<String> {
	let str = normalize(str);

	if str.is_empty() {
		None
	} else {
		Some(str)
	}
}
