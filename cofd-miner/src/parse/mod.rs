use std::{collections::HashMap, ops::Range};

use anyhow::Result;
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use cofd_meta::PageKind;
use cofd_schema::{
	book::{Book, BookInfo, BookReference},
	item::{gift::GiftKind, ActionFields},
};

mod gift;
mod item;
mod merit;

use crate::source::Section;

use self::{
	gift::parse_gifts,
	item::{convert_dice_pool, ItemProp},
	merit::parse_merits,
};

lazy_static! {
	static ref PROP_REGEX: Regex = Regex::new(
		r"^(Prerequisite|Style Tag|Cost|Dice Pool|Action|Duration|Effect|Drawback|Note|Exceptional Success|Success|Failure|Dramatic Failure)s?:\s?(.*)$"
	)
	.unwrap();
}

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
			match &section.kind {
				PageKind::Merit(_) => parse.merits.extend(parse_merits(&parse.info, &section)?),
				// PageKind::MageSpell => parse.mage_spells.extend(vec.into_iter().map(|i| match i {
				// 	_ => unreachable!(),
				// })),
				PageKind::Gift(kind) => match kind {
					// GiftKind::Moon => parse.moon_gifts.extend(todo!()),
					GiftKind::Shadow | GiftKind::Wolf => {
						parse.gifts.extend(parse_gifts(&parse.info, &section)?);
					}
					_ => {}
				},
				_ => {}
			}
		}
		parse.merits.sort_by(|a, b| a.name.cmp(&b.name));
		parse.mage_spells.sort_by(|a, b| a.name.cmp(&b.name));

		Ok(parse)
	}
}

fn get_book_reference(
	captures: &Captures<'_>,
	section: &Section,
	info: &BookInfo,
) -> BookReference {
	let page = if let Some(match_) = captures.get(0) {
		get_page_number(&section.page_ranges, match_.start())
	} else {
		unreachable!()
	};
	BookReference(info.id, *page)
}

fn process_action(action: &mut Option<ActionFields>, prop_key: ItemProp, lines: Vec<String>) {
	match prop_key {
		ItemProp::Action => action.get_or_insert_with(ActionFields::default).action = lines,
		ItemProp::Cost => action.get_or_insert_with(ActionFields::default).cost = lines,
		ItemProp::DicePool => {
			action.get_or_insert_with(ActionFields::default).dice_pool = convert_dice_pool(&lines);
		}
		ItemProp::Duration => action.get_or_insert_with(ActionFields::default).duration = lines,
		ItemProp::DramaticFailure => {
			action
				.get_or_insert_with(ActionFields::default)
				.roll_results
				.dramatic_failure = to_paragraphs(lines);
		}
		ItemProp::Failure => {
			action
				.get_or_insert_with(ActionFields::default)
				.roll_results
				.failure = to_paragraphs(lines);
		}
		ItemProp::Success => {
			action
				.get_or_insert_with(ActionFields::default)
				.roll_results
				.success = to_paragraphs(lines);
		}
		ItemProp::ExceptionalSuccess => {
			action
				.get_or_insert_with(ActionFields::default)
				.roll_results
				.exceptional_success = to_paragraphs(lines);
		}
		_ => {}
	}
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
		name.to_owned()
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
