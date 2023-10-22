use std::{collections::BTreeMap, str::FromStr};

use convert_case::{Case, Casing};
use either::Either;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use crate::source::Section;
use cofd_meta_schema::PageKind;
use cofd_schema::{
	book::{Book, BookInfo},
	dice_pool::DicePool,
	item::{merit::MeritTag, Item, ItemProp, PropValue, SubItem},
	prelude::DotRange,
	prerequisites::Prerequisite,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfExtract {
	pub info: BookInfo,
	pub sections: Vec<Section>,
	// errors: Vec<Error>,
}

fn convert_properties(
	item_or_properties: &mut Either<&mut Item, &mut BTreeMap<ItemProp, PropValue>>,
) {
	for (prop, value) in match item_or_properties {
		Either::Left(item) => &mut item.properties,
		Either::Right(properties) => properties,
	} {
		match prop {
			ItemProp::Prerequisites => {
				if let PropValue::Vec(vec) = value {
					let mut prereqs = Vec::new();

					for str in vec {
						if let Ok(prereq) = Prerequisite::from_str(str) {
							prereqs.push(prereq);
						}
					}

					if !prereqs.is_empty() {
						*value = PropValue::Prerequisites(prereqs);
					}
				}
			}
			ItemProp::Tags => {
				if let PropValue::Vec(vec) = value {
					let mut tags = Vec::new();

					for str in vec {
						if let Ok(prereq) = MeritTag::from_str(str) {
							tags.push(prereq);
						}
					}

					if !tags.is_empty() {
						*value = PropValue::Tags(tags);
					}
				}
			}
			ItemProp::DicePool => {
				if let PropValue::Vec(vec) = value {
					if vec.len() == 1 {
						let str = vec.first().unwrap();

						if let Ok(pool) = DicePool::from_str(str) {
							*value = PropValue::DicePool(pool);
						}
					}
				}
			}
			_ => {}
		}
	}
	if let Either::Left(item) = item_or_properties {
		for child in &mut item.children {
			convert_properties(&mut Either::Right(&mut child.properties));
		}
	}
}

impl PdfExtract {
	pub fn parse(self) -> Book {
		let mut parse = Book {
			info: self.info,
			merits: Default::default(),
			mage_spells: Default::default(),
		};

		let sections: Vec<(PageKind, Vec<Item>)> = self
			.sections
			.par_iter()
			.map(|span| (span.kind.clone(), span.parse()))
			.map(|(kind, mut parsed)| {
				parsed
					.par_iter_mut()
					.for_each(|item| convert_properties(&mut Either::Left(item)));

				(kind, parsed)
			})
			.collect();

		for (kind, vec) in sections {
			match kind {
				PageKind::Merit(_) => parse.merits.extend(vec),
				PageKind::MageSpell => parse.mage_spells.extend(vec),
			}
		}
		parse.merits.sort_by(|a, b| a.name.cmp(&b.name));
		parse.mage_spells.sort_by(|a, b| a.name.cmp(&b.name));

		parse
	}
}

lazy_static! {
	static ref MERIT_HEADER_REGEX: Regex = Regex::new(r"(?xmi)
		^
		(?P<name>[^\s.][^\n.]+)               # Name
		\s?
		\(
			(?: (?P<ltags> [^•\n]+ ) [,;] \s)?       # Tags
			(?P<cost>                         # Cost
				(?:          
					•{1,5}
					[,\s\+]*

					(?:to|or)?
					\s*
				)+
			)
			(?: [,;] \s (?P<rtags> [^•\n]+ ) )? # Tags
		\)
		( (?P<sub>:) \s (?P<subbegin> .* ) )?
		\s?
		$
	").unwrap();

	static ref PROP_REGEX: Regex = Regex::new(r"^(Prerequisite|Style Tag|Cost|Dice Pool|Action|Duration|Effect|Drawback|Note)s?:\s?(.*)$").unwrap();
	//
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

pub fn parse(page_kind: &PageKind, span: &str) -> Vec<Item> {
	let mut out = Vec::new();
	let mut str_pos = span.len();

	let mut children: Vec<SubItem> = Vec::new();

	let matches: Vec<_> = match page_kind {
		PageKind::Merit(_) => MERIT_HEADER_REGEX.captures_iter(span).collect(),
		PageKind::MageSpell => vec![],
	};
	for captures in matches.into_iter().rev() {
		let mut props = BTreeMap::new();

		let name = normalize(captures.name("name").unwrap().as_str());
		let name = if name.chars().all(|f| f.is_uppercase() || !f.is_alphabetic()) {
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
					name.replace_range(i..(i + 1), "-");
				}
				name
			}
		} else {
			name.to_string()
		};

		let desc = match page_kind {
			PageKind::Merit(additional_prereqs) => {
				let cost = captures.name("cost");

				let sub = captures.name("sub").is_some();
				let sub_begin = captures.name("subbegin");

				let ltags = captures
					.name("ltags")
					.map(|m| m.as_str().to_case(Case::Title));
				let rtags = captures
					.name("rtags")
					.map(|m| m.as_str().to_case(Case::Title));

				if let Some(tags) = ltags.or(rtags) {
					props.insert(
						ItemProp::Tags,
						PropValue::Vec(tags.split(", ").map(String::from).collect()),
					);
				}

				if let Some(cost) = cost {
					props.insert(
						ItemProp::DotRating,
						PropValue::DotRange(DotRange::from_str(cost.as_str()).unwrap()),
					);
				}

				if let Some(prereqs) = additional_prereqs {
					if !sub {
						props
							.entry(ItemProp::Prerequisites)
							.or_insert(PropValue::Vec(Vec::new()))
							.insert(0, prereqs.clone());
					}
				}

				let desc = {
					let body = get_body(&mut str_pos, span, &captures);

					let mut lines: Vec<String> = Vec::new();
					for el in body.iter().rev() {
						if let Some(prop) = PROP_REGEX.captures(el) {
							let prop_key =
								ItemProp::from_str(prop.get(1).unwrap().as_str()).unwrap();

							let prop_val = prop.get(2).unwrap().as_str().to_owned();

							lines.push(prop_val);
							lines.reverse();
							match prop_key {
								ItemProp::Prerequisites => {
									props.insert(
										ItemProp::Prerequisites,
										PropValue::Vec(
											to_paragraphs(lines)
												.join(" ")
												.split(", ")
												.map(str::to_owned)
												.collect(),
										),
									);
								}
								_ => {
									props.insert(prop_key, PropValue::Vec(to_paragraphs(lines)));
								}
							}
							lines = Vec::new();
						} else {
							lines.push(el.clone());
						}
					}
					lines.reverse();
					lines
				};

				if sub {
					let mut desc = desc;
					desc.insert(0, normalize(sub_begin.unwrap().as_str()));

					children.push(SubItem {
						name: name.to_owned(),
						description: to_paragraphs(desc),
						properties: props,
					});
					continue;
				}

				desc
			}

			PageKind::MageSpell => {
				vec![]
			}
		};

		children.reverse();
		out.push(Item {
			name: name.to_owned(),
			description: to_paragraphs(desc),
			children,
			properties: props,
		});
		children = Vec::new();
	}
	out
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