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
use crate::parse::paragraph::to_paragraphs;

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
	#[warn(clippy::match_wildcard_for_single_variants)]
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

mod paragraph {
	// TODO: Some edge cases don't get merged properly but works ok overall.
	fn to_paragraphs_old(vec: Vec<String>) -> Vec<String> {
		let mut out = Vec::new();
		let mut paragraph = String::new();

		let mut flag = false;
		for line in vec {
			if !paragraph.is_empty() && !flag {
				paragraph.push(' ');
			} else if flag {
				flag = false;
			}

			let line = line.trim_start();
			let line = if line.ends_with("God-") {
				flag = true;
				line
			} else if line.ends_with('-') {
				flag = true;
				line.trim_end_matches('-')
			} else {
				line
			};

			paragraph.push_str(line);

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

	fn to_paragraphs_tab(lines: Vec<String>) -> Vec<String> {
		let mut paragraphs = Vec::new();
		let mut paragraph = Vec::new();

		for line in lines.into_iter().rev() {
			let f = line.starts_with('\t');

			let line = line.trim_start();
			let line = if line.ends_with("God-") {
				line
			} else if line.ends_with('-') {
				line.trim_end_matches('-')
			} else {
				line
			}
			.to_owned();
			paragraph.push(line);

			if f {
				paragraph.reverse();
				paragraphs.push(paragraph.concat().trim().to_owned());
				paragraph = Vec::new();
			}
		}

		if !paragraph.is_empty() {
			paragraph.reverse();
			paragraphs.push(paragraph.concat().trim().to_owned());
		}

		paragraphs.reverse();
		paragraphs
	}

	pub fn to_paragraphs(mut lines: Vec<String>) -> Vec<String> {
		if lines.len() == 1 {
			let line = lines.pop().unwrap();
			let line = line.trim();

			vec![line.to_owned()]
		} else {
			let count = lines.iter().filter(|l| l.starts_with('\t')).count();

			if count > (lines.len() / 2) {
				to_paragraphs_old(lines)
			} else {
				to_paragraphs_tab(lines)
			}
		}
	}
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
	let name = normalize(captures.name("name").map_or("", |f| f.as_str().trim())).replace('\t', "");

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
		name.clone()
	}
}

fn normalize(str: &str) -> String {
	str
		// .trim_end()
		.replace('\n', "")
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

#[cfg(test)]
mod test {
	use crate::parse::paragraph::to_paragraphs;

	#[test]
	fn paragraph_test() {
		let text = vec![
			"STRINGS OF THE HEART (••)",
			"\tPrerequisites: Storm Lord",
			"\tEffect: The first trick to making someone do what you want ",
			"is finding out what they want, and promising it, threatening ",
			"it, or offering it. Your Storm Lord has a knack for finding that ",
			"very thing. After a turn scrutinizing her prey, ask his player, ",
			"“What does your character want most?” Your Storm Lord ",
			"instinctively knows the answer, even if she doesn’t understand ",
			"the context. “I want Davis’s hand in marriage” is more useful ",
			"if she knows who Davis is, but she doesn’t have to know him ",
			"to know that answer.",
			"\tWhen leveraging that bit of information, she’s considered",
			"one stage of impression better in Social maneuvers against",
			"the prey (see p. 163), and ignores one Door. As well, the prey",
			"cannot defy the Storm Lord’s threats, offers, or temptations",
			"without spending a point of Willpower.",
			"\tDrawback: That degree of intimacy creates a lasting",
			"relationship between the Storm Lord and her prey, whether",
			"she wants it or not. That sympathy leaves her open to later",
			"influence. The Storm Lord always has one fewer Door when",
			"the prey initiates a Social maneuver against her.",
		];
		let para = to_paragraphs(
			text.into_iter()
				.map(std::borrow::ToOwned::to_owned)
				.collect(),
		);

		for para in para {
			println!("Paragraph: {para}");
		}
	}
}
